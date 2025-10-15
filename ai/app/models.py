"""Model management and registry for the fruit detection service."""

import json
import logging
import os
from pathlib import Path
from typing import Dict, Optional, Tuple

import cv2
import numpy as np
import torch
import torch.nn as nn
from torchvision import models

from app.config import config
from app.exceptions import ModelLoadError
from app.logging_config import get_logger
from app.model_metadata import metadata_manager
from app.model_cache import model_cache
from app.utils.preprocess import preprocess_for_inference

logger = get_logger(__name__)


class BaseModel:
    """Base class for models in the registry."""
    
    def __init__(self, name: str, version: str = "1.0.0"):
        self.name = name
        self.version = version
        self.model = None
        self.labels = {}
    
    def load(self):
        """Load the model - to be implemented by subclasses."""
        raise NotImplementedError
    
    def predict(self, image: np.ndarray):
        """Run prediction on an image - to be implemented by subclasses."""
        raise NotImplementedError


class DenseNetModel(BaseModel):
    """DenseNet-based fruit detection model."""
    
    def __init__(self, name: str, version: str = "1.0.0"):
        super().__init__(name, version)
        self.device = torch.device("cuda" if config.USE_GPU and torch.cuda.is_available() else "cpu")
        self.num_classes = 0
        self.top_k = min(config.TOP_K, 5)  # Limit top_k to reasonable values
        self.metadata = metadata_manager.get_metadata(name)
    
    def load(self):
        """Load the DenseNet model."""
        logger.info(f"Loading {self.name} model version {self.version}")
        
        # Load labels
        self.labels = self._load_labels()
        self.num_classes = len(self.labels)
        
        # Update model metadata
        self.metadata.update_num_classes(self.num_classes)
        self.metadata.update_classes(list(self.labels.values()))
        
        if self.num_classes == 0:
            raise ModelLoadError("No class labels available for model initialization")
        
        # Initialize model
        logger.info("Initializing DenseNet121 model")
        try:
            # Initialize model without pretrained weights for faster startup
            # We'll load our custom weights right after
            self.model = models.densenet121(weights=None)
            features = self.model.classifier.in_features
            self.model.classifier = nn.Linear(features, self.num_classes)
        except Exception as e:
            logger.error(f"Failed to initialize model: {e}")
            raise ModelLoadError(f"Failed to initialize model: {e}") from e
        
        # Load custom weights if available
        weights_path = config.MODEL_PATH
        if os.path.exists(weights_path):
            try:
                state_dict = torch.load(weights_path, map_location=self.device)
                # Try to load the complete state dict directly
                try:
                    self.model.load_state_dict(state_dict, strict=True)
                    logger.info(
                        "Loaded complete weights from %s (all %d layers)",
                        weights_path,
                        len(state_dict),
                    )
                except RuntimeError as e:
                    # If strict loading fails, fallback to partial loading
                    logger.warning(f"Strict loading failed: {e}. Attempting partial loading...")
                    model_dict = self.model.state_dict()
                    filtered_dict = {
                        k: v
                        for k, v in state_dict.items()
                        if k in model_dict and v.shape == model_dict[k].shape
                    }
                    model_dict.update(filtered_dict)
                    self.model.load_state_dict(model_dict)
                    logger.info(
                        "Loaded partial weights from %s (%d/%d layers)",
                        weights_path,
                        len(filtered_dict),
                        len(state_dict),
                    )
            except Exception as e:
                logger.warning(f"Failed to load custom weights: {e}. Using pre-trained backbone only.")
        else:
            logger.warning(
                "Custom weights not found at %s. Using pre-trained backbone with random classifier",
                weights_path
            )
        
        try:
            self.model.to(self.device)
            self.model.eval()
            logger.info(f"Model {self.name} loaded successfully on {self.device}")
        except Exception as e:
            logger.error(f"Failed to move model to device {self.device}: {e}")
            raise ModelLoadError(f"Failed to move model to device: {e}") from e
    
    def _load_labels(self) -> Dict[int, str]:
        """Load class labels."""
        # First, try to infer the number of classes from the weights file
        weights_path = Path(config.MODEL_PATH)
        expected_classes = None

        if weights_path.exists():
            try:
                state_dict = torch.load(weights_path, map_location='cpu')
                if 'classifier.weight' in state_dict:
                    expected_classes = state_dict['classifier.weight'].shape[0]
                    logger.info(f"Detected {expected_classes} classes from model weights")
            except Exception as e:
                logger.warning(f"Could not read class count from weights: {e}")

        # Try to load from labels.json first (most reliable)
        try:
            labels_path = Path(config.LABELS_PATH)
            if labels_path.exists():
                with open(labels_path, 'r') as f:
                    labels_dict = json.load(f)
                    dataset_labels = {int(k): v for k, v in labels_dict.items()}

                    # Verify the number matches expected classes
                    if expected_classes and len(dataset_labels) != expected_classes:
                        logger.warning(
                            f"labels.json has {len(dataset_labels)} classes "
                            f"but model expects {expected_classes}"
                        )
                    else:
                        logger.info(f"Loaded {len(dataset_labels)} class labels from {labels_path}")
                        return dataset_labels
        except (FileNotFoundError, json.JSONDecodeError, ValueError) as e:
            logger.warning(f"Failed to load labels from labels.json: {e}")

        # Try to load from Test directory
        test_dir = Path(config.TEST_DATA_DIR)

        dataset_labels: Dict[int, str] = {}
        if test_dir.exists() and test_dir.is_dir():
            folder_names = sorted(
                entry.name for entry in test_dir.iterdir() if entry.is_dir()
            )
            if folder_names:
                dataset_labels = {index: name for index, name in enumerate(folder_names)}

                # Only use if it matches expected classes
                if expected_classes is None or len(dataset_labels) == expected_classes:
                    self._persist_labels(dataset_labels)
                    logger.info(
                        "Loaded %s class labels from test folder %s",
                        len(dataset_labels),
                        test_dir,
                    )
                    return dataset_labels
                else:
                    logger.warning(
                        f"Test directory has {len(dataset_labels)} classes "
                        f"but model expects {expected_classes}"
                    )
            else:
                logger.warning("Test directory %s has no class folders", test_dir)

        # Fallback to Training directory
        training_dir = Path(config.TRAINING_DATA_DIR)
        if training_dir.exists() and training_dir.is_dir():
            folder_names = sorted(
                entry.name for entry in training_dir.iterdir() if entry.is_dir()
            )
            if folder_names:
                dataset_labels = {index: name for index, name in enumerate(folder_names)}
                self._persist_labels(dataset_labels)
                logger.info(
                    "Loaded %s class labels from training folder %s",
                    len(dataset_labels),
                    training_dir,
                )
                return dataset_labels
            logger.warning("Training directory %s has no class folders", training_dir)

        # Fallback to labels.json
        try:
            with open(config.LABELS_PATH, "r", encoding="utf-8") as file:
                raw_labels = json.load(file)

            # Handle different JSON formats
            if isinstance(raw_labels, dict):
                if "labels" in raw_labels:
                    # Format: {"labels": ["Class1", "Class2", ...]}
                    labels_list = raw_labels["labels"]
                    labels = {i: name for i, name in enumerate(labels_list)}
                else:
                    # Format: {"0": "Class1", "1": "Class2", ...}
                    labels = {int(k): v for k, v in raw_labels.items()}
            elif isinstance(raw_labels, list):
                # Format: ["Class1", "Class2", ...]
                labels = {i: name for i, name in enumerate(raw_labels)}
            else:
                raise ValueError(f"Unexpected labels format: {type(raw_labels)}")

            logger.info("Loaded %s class labels from labels.json", len(labels))
            return labels
        except FileNotFoundError:
            logger.warning("Labels file not found, using default fallback labels")
            return {
                0: "apple",
                1: "banana",
                2: "orange",
                3: "grape",
                4: "strawberry",
                5: "pineapple",
                6: "watermelon",
                7: "peach",
            }
    
    def _persist_labels(self, labels: Dict[int, str]) -> None:
        """Persist labels to file."""
        try:
            labels_path = Path(config.LABELS_PATH)
            labels_path.parent.mkdir(parents=True, exist_ok=True)
            with labels_path.open("w", encoding="utf-8") as file:
                json.dump({str(k): v for k, v in labels.items()}, file, ensure_ascii=False, indent=2)
        except OSError as exc:
            logger.error("Failed to persist labels.json: %s", exc)
    
    def predict(self, image: np.ndarray) -> Tuple[np.ndarray, np.ndarray]:
        """Run prediction on an image."""
        if self.model is None or self.num_classes == 0:
            raise RuntimeError("Model not loaded for inference")

        # Use centralized preprocessing function
        image_tensor = preprocess_for_inference(image, target_size=config.IMG_SIZE).unsqueeze(0).to(self.device)

        with torch.no_grad():
            logits = self.model(image_tensor)
            probabilities = torch.softmax(logits, dim=1)
            top_probs, top_indices = probabilities.topk(self.top_k, dim=1)

        return top_probs[0].cpu().numpy(), top_indices[0].cpu().numpy()


class ModelRegistry:
    """Registry to manage multiple models."""
    
    def __init__(self):
        self.models: Dict[str, BaseModel] = {}
        self.default_model_name = config.DEFAULT_MODEL_NAME
    
    def register_model(self, model: BaseModel):
        """Register a model in the registry."""
        self.models[model.name] = model
        logger.info(f"Registered model: {model.name} (version {model.version})")
    
    def load_model(self, model_name: str) -> BaseModel:
        """Load and return a model by name with caching."""
        # First check the cache
        cached_model = model_cache.get_model(model_name)
        if cached_model is not None:
            return cached_model
        
        if model_name not in self.models:
            if model_name == config.DEFAULT_MODEL_NAME:
                # Create and register the default DenseNet model
                model = DenseNetModel(model_name, config.MODEL_VERSION)
                self.register_model(model)
            else:
                raise ModelLoadError(f"Unknown model: {model_name}")
        
        model = self.models[model_name]
        if model.model is None:
            model.load()
        
        # Cache the model for future use
        model_cache.set_model(model_name, model)
        
        return model
    
    def get_model(self, model_name: Optional[str] = None) -> BaseModel:
        """Get a loaded model by name or return the default model."""
        if model_name is None:
            model_name = self.default_model_name
        
        return self.load_model(model_name)
    
    def list_models(self) -> list:
        """List all registered model names."""
        return list(self.models.keys())


# Create a global model registry instance
model_registry = ModelRegistry()