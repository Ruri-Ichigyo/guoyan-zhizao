import cv2
import numpy as np
import torch
from typing import Optional, Tuple

from app.config import config
from app.logging_config import get_logger
from app.exceptions import ImageProcessingError

logger = get_logger(__name__)


def preprocess_image(
    image: np.ndarray
) -> torch.Tensor:
    """
    Preprocess an image for model inference.

    Args:
        image: Input image in BGR format.

    Returns:
        A normalized tensor ready for model inference.
    """
    if image is None or image.size == 0:
        raise ValueError("Input image is empty.")

    # Convert BGR numpy array to RGB and resize
    rgb_image = cv2.cvtColor(image, cv2.COLOR_BGR2RGB)
    resized_image = cv2.resize(rgb_image, config.IMG_SIZE)
    
    # Convert to tensor and normalize
    tensor = torch.from_numpy(resized_image).float() / 255.0
    tensor = tensor.permute(2, 0, 1)  # HWC to CHW
    
    # Apply configured normalization
    mean = torch.tensor(config.IMG_NORMALIZATION_MEAN).view(3, 1, 1)
    std = torch.tensor(config.IMG_NORMALIZATION_STD).view(3, 1, 1)
    tensor = (tensor - mean) / std
    
    return tensor


def letterbox(image: np.ndarray, new_shape: Tuple[int, int], color=(114, 114, 114)) -> np.ndarray:
    """
    Resize image with letterboxing to maintain aspect ratio.

    Args:
        image: Input image
        new_shape: Target shape (height, width)
        color: Fill color for letterbox padding

    Returns:
        Resized image with letterboxing
    """
    # Current shape [height, width]
    shape = image.shape[:2]

    # Scale ratio (new / old)
    r = min(new_shape[0] / shape[0], new_shape[1] / shape[1])

    # New unpadded size
    new_unpad = (int(round(shape[1] * r)), int(round(shape[0] * r)))

    # Padding
    dw = (new_shape[1] - new_unpad[0]) / 2
    dh = (new_shape[0] - new_unpad[1]) / 2

    if shape[::-1] != new_unpad:  # resize
        image = cv2.resize(image, new_unpad, interpolation=cv2.INTER_LINEAR)

    # Pad to new_shape
    top, bottom = int(round(dh - 0.1)), int(round(dh + 0.1))
    left, right = int(round(dw - 0.1)), int(round(dw + 0.1))

    image = cv2.copyMakeBorder(
        image, top, bottom, left, right, cv2.BORDER_CONSTANT, value=color
    )

    return image


def enhance_image_quality(image: np.ndarray) -> np.ndarray:
    """
    Enhance image quality for better detection.

    Args:
        image: Input image

    Returns:
        Enhanced image
    """
    # Convert to LAB color space
    lab = cv2.cvtColor(image, cv2.COLOR_RGB2LAB)

    # Split channels
    l, a, b = cv2.split(lab)

    # Apply CLAHE (Contrast Limited Adaptive Histogram Equalization) to L channel
    clahe = cv2.createCLAHE(clipLimit=2.0, tileGridSize=(8, 8))
    l = clahe.apply(l)

    # Merge channels and convert back to RGB
    lab = cv2.merge([l, a, b])
    enhanced = cv2.cvtColor(lab, cv2.COLOR_LAB2RGB)

    # Apply slight gaussian blur to reduce noise
    enhanced = cv2.GaussianBlur(enhanced, (3, 3), 0)

    return enhanced


def adjust_brightness_contrast(image: np.ndarray, brightness: float = 0, contrast: float = 1.0) -> np.ndarray:
    """
    Adjust image brightness and contrast.

    Args:
        image: Input image
        brightness: Brightness adjustment (-100 to 100)
        contrast: Contrast adjustment (0.5 to 3.0)

    Returns:
        Adjusted image
    """
    adjusted = cv2.convertScaleAbs(image, alpha=contrast, beta=brightness)
    return adjusted


def preprocess_for_inference(
    image: np.ndarray,
    target_size: Optional[Tuple[int, int]] = None,
    normalize: bool = True
) -> torch.Tensor:
    """
    Flexible preprocessing function for model inference.

    Args:
        image: Input image in BGR format
        target_size: Target size for resizing (height, width). If None, uses config.IMG_SIZE
        normalize: Whether to apply normalization

    Returns:
        Preprocessed tensor
    """
    if image is None or image.size == 0:
        raise ValueError("Input image is empty.")

    # Use config IMG_SIZE if target_size not provided
    target_size = target_size or config.IMG_SIZE

    # Convert BGR numpy array to RGB and resize
    rgb_image = cv2.cvtColor(image, cv2.COLOR_BGR2RGB)
    resized_image = cv2.resize(rgb_image, target_size)
    
    # Convert to tensor and normalize
    tensor = torch.from_numpy(resized_image).float() / 255.0
    tensor = tensor.permute(2, 0, 1)  # HWC to CHW
    
    if normalize:
        # Apply configured normalization
        mean = torch.tensor(config.IMG_NORMALIZATION_MEAN).view(3, 1, 1)
        std = torch.tensor(config.IMG_NORMALIZATION_STD).view(3, 1, 1)
        tensor = (tensor - mean) / std
    
    return tensor
