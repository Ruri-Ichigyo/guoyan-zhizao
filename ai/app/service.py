from datetime import datetime, timezone
from typing import Dict, List, Optional

import cv2
import numpy as np

from app.config import config
from app.exceptions import ImageProcessingError, InferenceError
from app.models import model_registry
from app.schemas import DetectionResult, InferenceResponse
from app.utils.postprocess import calculate_quality_grade
from app.utils.preprocess import preprocess_image
from app.logging_config import get_logger

logger = get_logger(__name__)


class FruitDetectionService:
    """Wraps the model loading and inference workflow for fruit detection."""

    def __init__(self, model_name: Optional[str] = None) -> None:
        self.model_name = model_name or config.DEFAULT_MODEL_NAME
        self.model = model_registry.get_model(self.model_name)

    def _decode_image(self, data: bytes) -> np.ndarray:
        """Decode image bytes to numpy array."""
        try:
            array = np.frombuffer(data, np.uint8)
            image = cv2.imdecode(array, cv2.IMREAD_COLOR)
            if image is None:
                raise ImageProcessingError("Failed to decode image data")
            return image
        except Exception as e:
            logger.error("Image decoding failed: %s", str(e))
            raise ImageProcessingError(f"Failed to decode image data: {str(e)}") from e

    def infer(self, upload_id: str, file_bytes: bytes, model_name: Optional[str] = None) -> InferenceResponse:
        """Perform inference on an image."""
        try:
            if len(file_bytes) == 0:
                raise ImageProcessingError("Empty file received")

            # Use specific model if provided, otherwise use default
            current_model = model_registry.get_model(model_name) if model_name else self.model
            logger.info("Starting inference for upload_id: %s with model: %s", upload_id, current_model.name)

            image = self._decode_image(file_bytes)
            detections = self._detect(image, current_model)

            response = InferenceResponse(
                upload_id=upload_id,
                status="completed",
                total_fruits=len(detections),
                timestamp=datetime.now(timezone.utc),
                results=[DetectionResult(**item) for item in detections],
                metadata={
                    "model": current_model.name,
                    "model_version": current_model.version,
                    "device": str(current_model.device),
                    "top_k": str(current_model.top_k),
                },
            )

            logger.info("Inference completed for upload_id: %s, found %d detections", upload_id, len(detections))
            return response
        except ImageProcessingError:
            # Re-raise known image processing errors
            raise
        except Exception as e:
            logger.error("Inference failed for upload_id %s: %s", upload_id, str(e), exc_info=True)
            raise InferenceError(f"Inference failed: {str(e)}") from e

    def _detect(self, image: np.ndarray, model) -> List[Dict]:
        """Detect fruits in an image using the specified model."""
        if model.model is None or model.num_classes == 0:
            raise InferenceError("Model not ready for inference")

        try:
            # Get predictions from the model
            top_probs, top_indices = model.predict(image)
            
            detections: List[Dict] = []
            height, width = image.shape[:2]

            for i in range(len(top_probs)):
                probability = top_probs[i]
                class_index = top_indices[i]
                
                confidence = float(probability)
                class_id = int(class_index)
                class_name = model.labels.get(class_id, f"unknown_{class_id}")

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
        except Exception as e:
            logger.error("Detection failed: %s", str(e), exc_info=True)
            raise InferenceError(f"Detection failed: {str(e)}") from e
