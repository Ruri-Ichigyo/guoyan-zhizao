"""Configuration management for the fruit detection service."""

import os
from dataclasses import dataclass, field


@dataclass
class Config:
    """Configuration class for the fruit detection service."""

    # Model settings
    MODEL_PATH: str = os.getenv("MODEL_PATH", "model/fruit_densenet_weights.pth")
    LABELS_PATH: str = os.getenv("LABELS_PATH", "model/labels.json")
    DEFAULT_MODEL_NAME: str = os.getenv("DEFAULT_MODEL_NAME", "densenet121-fruit-v1")

    # API settings
    API_HOST: str = os.getenv("API_HOST", "0.0.0.0")
    API_PORT: int = int(os.getenv("API_PORT", "8002"))

    # Image processing settings
    IMG_SIZE: tuple = field(default_factory=lambda: tuple(
        map(int, os.getenv("IMG_SIZE", "224,224").split(","))
    ))
    IMG_NORMALIZATION_MEAN: list = field(default_factory=lambda: list(
        map(float, os.getenv("IMG_NORMALIZATION_MEAN", "0.485,0.456,0.406").split(","))
    ))
    IMG_NORMALIZATION_STD: list = field(default_factory=lambda: list(
        map(float, os.getenv("IMG_NORMALIZATION_STD", "0.229,0.224,0.225").split(","))
    ))

    # Device settings
    USE_GPU: bool = os.getenv("USE_GPU", "True").lower() in ("true", "1", "yes")

    # Quality assessment settings
    QUALITY_ASSESSMENT_ENABLED: bool = os.getenv("QUALITY_ASSESSMENT_ENABLED", "True").lower() in ("true", "1", "yes")

    # Performance settings
    BATCH_SIZE: int = int(os.getenv("BATCH_SIZE", "1"))
    TOP_K: int = int(os.getenv("TOP_K", "3"))

    # File paths
    TEST_DATA_DIR: str = os.getenv("TEST_DATA_DIR", "Test")
    TRAINING_DATA_DIR: str = os.getenv("TRAINING_DATA_DIR", "Training")

    # Debug settings
    DEBUG: bool = os.getenv("DEBUG", "False").lower() in ("true", "1", "yes")

    # Model metadata
    MODEL_VERSION: str = os.getenv("MODEL_VERSION", "v1.0.0")
    SERVICE_VERSION: str = os.getenv("SERVICE_VERSION", "0.2.0")


# Create a global config instance
config = Config()