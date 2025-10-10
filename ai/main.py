import json
import logging
import os
from datetime import datetime
from pathlib import Path
from typing import Dict, List

import cv2
import numpy as np
import torch
import torch.nn as nn
from fastapi import FastAPI, File, Form, HTTPException, UploadFile
from fastapi.middleware.cors import CORSMiddleware
from pydantic import BaseModel
from torchvision import models

from utils.postprocess import calculate_quality_grade
from utils.preprocess import DEFAULT_TRANSFORM, preprocess_image

logging.basicConfig(
    level=logging.INFO,
    format="%(asctime)s - %(name)s - %(levelname)s - %(message)s",
)
logger = logging.getLogger(__name__)


class DetectionResult(BaseModel):
    class_name: str
    confidence: float
    bbox: List[float]
    quality_grade: str


class InferenceResponse(BaseModel):
    upload_id: str
    status: str
    total_fruits: int
    timestamp: datetime
    results: List[DetectionResult]
    metadata: Dict[str, str]


class FruitDetectionService:
    def __init__(self) -> None:
        self.device = torch.device("cuda" if torch.cuda.is_available() else "cpu")
        self.transform = DEFAULT_TRANSFORM
        self.labels = self._load_labels()
        self.num_classes = len(self.labels)
        self.model = self._load_model()
        self.top_k = min(3, self.num_classes) if self.num_classes else 0

    def _load_labels(self) -> Dict[int, str]:
        training_dir = Path(
            os.getenv(
                "TRAINING_DATA_DIR",
                "Fruit-Images-Dataset-master/Training",
            )
        )

        dataset_labels: Dict[int, str] = {}
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

        try:
            with open("model/labels.json", "r", encoding="utf-8") as file:
                raw_labels = json.load(file)
            labels = {int(k): v for k, v in raw_labels.items()}
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
        try:
            labels_path = Path("model/labels.json")
            labels_path.parent.mkdir(parents=True, exist_ok=True)
            with labels_path.open("w", encoding="utf-8") as file:
                json.dump({str(k): v for k, v in labels.items()}, file, ensure_ascii=False, indent=2)
        except OSError as exc:
            logger.error("Failed to persist labels.json: %s", exc)

    def _load_model(self) -> torch.nn.Module:
        if self.num_classes == 0:
            raise RuntimeError("No class labels available for model initialization")

        logger.info("Loading DenseNet121 model")
        model = models.densenet121(weights=models.DenseNet121_Weights.IMAGENET1K_V1)
        features = model.classifier.in_features
        model.classifier = nn.Linear(features, self.num_classes)

        weights_path = "model/fruit_densenet_weights.pth"
        if os.path.exists(weights_path):
            state_dict = torch.load(weights_path, map_location=self.device)
            # Load only the matching layers (feature extractor), skip classifier if shape mismatch
            model_dict = model.state_dict()
            filtered_dict = {k: v for k, v in state_dict.items()
                           if k in model_dict and v.shape == model_dict[k].shape}
            model_dict.update(filtered_dict)
            model.load_state_dict(model_dict)
            logger.info("Loaded custom weights from %s (%d/%d layers)",
                       weights_path, len(filtered_dict), len(state_dict))
        else:
            logger.warning(
                "Custom weights not found. Using pretrained backbone with random classifier"
            )

        model.to(self.device)
        model.eval()
        return model

    def _decode_image(self, data: bytes) -> np.ndarray:
        array = np.frombuffer(data, np.uint8)
        image = cv2.imdecode(array, cv2.IMREAD_COLOR)
        if image is None:
            raise ValueError("Failed to decode image data")
        return image

    def infer(self, upload_id: str, file_bytes: bytes) -> InferenceResponse:
        if len(file_bytes) == 0:
            raise ValueError("Empty file received")

        image = self._decode_image(file_bytes)
        detections = self._detect(image)

        response = InferenceResponse(
            upload_id=upload_id,
            status="completed",
            total_fruits=len(detections),
            timestamp=datetime.utcnow(),
            results=[DetectionResult(**item) for item in detections],
            metadata={
                "model": "densenet121-fruit-v1",
                "device": str(self.device),
                "top_k": str(self.top_k),
            },
        )
        return response

    def _detect(self, image: np.ndarray) -> List[Dict]:
        if self.model is None or self.top_k == 0:
            raise RuntimeError("Model not ready for inference")

        tensor = preprocess_image(image, self.transform).unsqueeze(0).to(self.device)

        with torch.no_grad():
            logits = self.model(tensor)
            probabilities = torch.softmax(logits, dim=1)
            top_probs, top_indices = probabilities.topk(self.top_k, dim=1)

        detections: List[Dict] = []
        height, width = image.shape[:2]

        for probability, class_index in zip(top_probs[0], top_indices[0]):
            confidence = float(probability.item())
            class_id = int(class_index.item())
            class_name = self.labels.get(class_id, f"unknown_{class_id}")

            quality_grade = calculate_quality_grade(image, class_name, confidence)

            detections.append(
                {
                    "class_name": class_name,
                    "confidence": confidence,
                    "bbox": [0.0, 0.0, float(width), float(height)],
                    "quality_grade": quality_grade,
                }
            )

        return detections


service = FruitDetectionService()

app = FastAPI(
    title="Fruit Detection Service",
    description="AI inference service for fruit sorting",
    version="0.2.0",
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)


@app.get("/health")
async def health() -> Dict[str, str]:
    return {"status": "ok"}


@app.post("/infer", response_model=InferenceResponse)
async def infer(upload_id: str = Form(...), file: UploadFile = File(...)) -> InferenceResponse:
    try:
        bytes_data = await file.read()
        logger.info("Received inference request for %s (%s bytes)", upload_id, len(bytes_data))
        return service.infer(upload_id, bytes_data)
    except ValueError as exc:
        logger.error("Validation error: %s", exc)
        raise HTTPException(status_code=400, detail=str(exc)) from exc
    except Exception as exc:  # pylint: disable=broad-except
        logger.exception("Inference failed: %s", exc)
        raise HTTPException(status_code=500, detail="Inference failed") from exc


if __name__ == "__main__":
    import uvicorn

    uvicorn.run(app, host="0.0.0.0", port=8002)
