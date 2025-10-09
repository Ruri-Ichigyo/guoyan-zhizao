import cv2
import numpy as np
import torch
from PIL import Image
from torchvision import transforms
from typing import Optional, Tuple

# Default preprocessing pipeline for DenseNet-style models
DEFAULT_TRANSFORM = transforms.Compose([
    transforms.Resize((224, 224)),  # Resize to DenseNet input resolution
    transforms.ToTensor(),          # Convert to tensor and scale to [0, 1]
    transforms.Normalize(           # Apply ImageNet normalization
        mean=[0.485, 0.456, 0.406],
        std=[0.229, 0.224, 0.225]
    ),
])

def preprocess_image(
    image: np.ndarray,
    transform: Optional[transforms.Compose] = None
) -> torch.Tensor:
    """
    Preprocess an image for DenseNet inference.

    Args:
        image: Input image in BGR format.
        transform: Optional torchvision transform pipeline. When None, uses
            DEFAULT_TRANSFORM.

    Returns:
        A normalized tensor ready for model inference.
    """
    if image is None or image.size == 0:
        raise ValueError("Input image is empty.")

    if transform is None:
        transform = DEFAULT_TRANSFORM

    # Convert BGR numpy array to PIL Image in RGB color space
    rgb_image = cv2.cvtColor(image, cv2.COLOR_BGR2RGB)
    pil_image = Image.fromarray(rgb_image)

    # Apply transformation pipeline
    tensor = transform(pil_image)

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
