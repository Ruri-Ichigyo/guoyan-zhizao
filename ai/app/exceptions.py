"""Custom exceptions for the fruit detection service."""

from typing import Optional


class FruitDetectionError(Exception):
    """Base exception class for fruit detection service errors."""
    
    def __init__(self, message: str, error_code: Optional[str] = None):
        self.message = message
        self.error_code = error_code
        super().__init__(self.message)


class ModelLoadError(FruitDetectionError):
    """Raised when model loading fails."""
    
    def __init__(self, message: str = "Model loading failed", error_code: str = "MODEL_LOAD_ERROR"):
        super().__init__(message, error_code)


class ImageProcessingError(FruitDetectionError):
    """Raised when image processing fails."""
    
    def __init__(self, message: str = "Image processing failed", error_code: str = "IMAGE_PROCESSING_ERROR"):
        super().__init__(message, error_code)


class InferenceError(FruitDetectionError):
    """Raised when inference fails."""
    
    def __init__(self, message: str = "Inference failed", error_code: str = "INFERENCE_ERROR"):
        super().__init__(message, error_code)


class QualityAssessmentError(FruitDetectionError):
    """Raised when quality assessment fails."""
    
    def __init__(self, message: str = "Quality assessment failed", error_code: str = "QUALITY_ASSESSMENT_ERROR"):
        super().__init__(message, error_code)