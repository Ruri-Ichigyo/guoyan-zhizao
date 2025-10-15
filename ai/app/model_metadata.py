"""Model metadata management for the fruit detection service."""

import json
import os
from datetime import datetime
from pathlib import Path
from typing import Dict, Optional

from app.config import config
from app.logging_config import get_logger

logger = get_logger(__name__)


class ModelMetadata:
    """Class to manage model metadata."""
    
    def __init__(self, model_name: str):
        self.model_name = model_name
        self.metadata_path = Path("model") / f"{model_name}_metadata.json"
        self.metadata = self._load_metadata()
    
    def _load_metadata(self) -> Dict:
        """Load model metadata from file or create default."""
        if self.metadata_path.exists():
            try:
                with open(self.metadata_path, 'r', encoding='utf-8') as f:
                    return json.load(f)
            except Exception as e:
                logger.warning(f"Failed to load metadata for {self.model_name}: {e}")
        
        # Create default metadata
        default_metadata = {
            "model_name": self.model_name,
            "version": config.MODEL_VERSION,
            "created_at": datetime.now().isoformat(),
            "framework": "torchvision",
            "architecture": "densenet121",
            "input_shape": [3, 224, 224],
            "num_classes": 0,
            "classes": [],
            "preprocessing": {
                "mean": config.IMG_NORMALIZATION_MEAN,
                "std": config.IMG_NORMALIZATION_STD,
                "size": config.IMG_SIZE
            }
        }
        
        return default_metadata
    
    def save_metadata(self):
        """Save model metadata to file."""
        try:
            self.metadata_path.parent.mkdir(parents=True, exist_ok=True)
            with open(self.metadata_path, 'w', encoding='utf-8') as f:
                json.dump(self.metadata, f, indent=2, ensure_ascii=False)
        except Exception as e:
            logger.error(f"Failed to save metadata for {self.model_name}: {e}")
    
    def update_num_classes(self, num_classes: int):
        """Update the number of classes in metadata."""
        self.metadata["num_classes"] = num_classes
        self.save_metadata()
    
    def update_classes(self, classes: list):
        """Update the class names in metadata."""
        self.metadata["classes"] = classes
        self.save_metadata()
    
    def get_metadata(self) -> Dict:
        """Get model metadata."""
        return self.metadata


class ModelMetadataManager:
    """Singleton class to manage all model metadata."""
    
    _instance = None
    _metadata_store: Dict[str, ModelMetadata] = {}
    
    def __new__(cls):
        if cls._instance is None:
            cls._instance = super(ModelMetadataManager, cls).__new__(cls)
        return cls._instance
    
    def get_metadata(self, model_name: str) -> ModelMetadata:
        """Get metadata for a specific model."""
        if model_name not in self._metadata_store:
            self._metadata_store[model_name] = ModelMetadata(model_name)
        return self._metadata_store[model_name]
    
    def list_metadata(self) -> Dict[str, Dict]:
        """List all model metadata."""
        result = {}
        for model_name, metadata_obj in self._metadata_store.items():
            result[model_name] = metadata_obj.get_metadata()
        return result


# Create global instance
metadata_manager = ModelMetadataManager()