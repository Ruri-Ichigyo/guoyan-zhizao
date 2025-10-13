"""App package for the fruit detection service."""

from app.config import config
from app.models import model_registry
from app.service import FruitDetectionService

__all__ = [
    "config", 
    "model_registry", 
    "FruitDetectionService"
]