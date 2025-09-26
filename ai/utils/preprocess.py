import cv2
import numpy as np
from typing import Tuple

def preprocess_image(image: np.ndarray, target_size: Tuple[int, int] = (640, 640)) -> np.ndarray:
    """
    Preprocess image for YOLO inference.

    Args:
        image: Input image in BGR format
        target_size: Target size for model input

    Returns:
        Preprocessed image
    """
    # Convert BGR to RGB
    rgb_image = cv2.cvtColor(image, cv2.COLOR_BGR2RGB)

    # Resize image while maintaining aspect ratio
    resized_image = letterbox(rgb_image, target_size)

    # Normalize pixel values
    normalized_image = resized_image.astype(np.float32) / 255.0

    return normalized_image

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