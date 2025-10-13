import cv2
import numpy as np
from typing import List, Dict, Tuple, Optional

from app.config import config
from app.logging_config import get_logger
from app.exceptions import QualityAssessmentError

logger = get_logger(__name__)


def postprocess_results(detections: List[Dict], image_shape: Tuple[int, int]) -> List[Dict]:
    """
    Post-process detection results.

    Args:
        detections: Raw detections from model
        image_shape: Original image shape (height, width)

    Returns:
        Processed detections with additional metadata
    """
    processed_detections = []

    for detection in detections:
        # Calculate relative coordinates
        x1, y1, x2, y2 = detection['bbox']
        height, width = image_shape[:2]

        relative_bbox = [
            x1 / width,   # x1_rel
            y1 / height,  # y1_rel
            x2 / width,   # x2_rel
            y2 / height   # y2_rel
        ]

        # Calculate area and aspect ratio
        area = (x2 - x1) * (y2 - y1)
        aspect_ratio = (x2 - x1) / (y2 - y1) if (y2 - y1) > 0 else 1.0

        processed_detection = {
            **detection,
            'relative_bbox': relative_bbox,
            'area': float(area),
            'aspect_ratio': float(aspect_ratio),
            'center': [(x1 + x2) / 2, (y1 + y2) / 2]
        }

        processed_detections.append(processed_detection)

    return processed_detections


def calculate_quality_grade(
    fruit_region: np.ndarray, 
    class_name: str, 
    confidence: float,
    target_class: Optional[str] = None
) -> str:
    """
    Calculate fruit quality grade based on visual features.

    Args:
        fruit_region: Cropped fruit image region
        class_name: Detected fruit class name
        confidence: Detection confidence
        target_class: Expected fruit class for more specific grading

    Returns:
        Quality grade: 'A', 'B', or 'C'
    """
    if not config.QUALITY_ASSESSMENT_ENABLED:
        # Return grade based on confidence only if quality assessment is disabled
        if confidence >= 0.9:
            return 'A'
        elif confidence >= 0.7:
            return 'B'
        else:
            return 'C'

    try:
        if fruit_region.size == 0:
            return 'C'

        # Convert to different color spaces for analysis
        hsv = cv2.cvtColor(fruit_region, cv2.COLOR_BGR2HSV)
        lab = cv2.cvtColor(fruit_region, cv2.COLOR_BGR2LAB)

        # Calculate various quality indicators
        color_uniformity = calculate_color_uniformity(hsv)
        defects = analyze_surface_defects(fruit_region)
        surface_quality = _estimate_surface_quality(defects, fruit_region.shape[:2])
        size_score = calculate_size_score(fruit_region.shape[:2])
        
        # Use class-specific freshness analysis if target class is known
        freshness_score = analyze_freshness(lab, target_class) if target_class else analyze_freshness(lab)

        # Adjust weights based on fruit type or other factors
        # For now, using general weights
        total_score = (
            confidence * 0.3 +
            color_uniformity * 0.25 +
            surface_quality * 0.25 +
            size_score * 0.1 +
            freshness_score * 0.1
        )

        # Determine grade based on total score
        if total_score >= 0.8:
            return 'A'
        elif total_score >= 0.6:
            return 'B'
        else:
            return 'C'

    except Exception as e:
        # Log the exception for debugging
        logger.warning("Quality assessment failed: %s", str(e))

        # Fallback to confidence-based grading
        if confidence >= 0.9:
            return 'A'
        elif confidence >= 0.7:
            return 'B'
        else:
            return 'C'


def calculate_color_uniformity(hsv_image: np.ndarray) -> float:
    """
    Calculate color uniformity score.

    Args:
        hsv_image: Image in HSV color space

    Returns:
        Uniformity score (0-1)
    """
    try:
        # Calculate standard deviation of hue and saturation
        h, s, v = cv2.split(hsv_image)

        # Focus on non-black pixels
        mask = v > 30

        if np.sum(mask) == 0:
            return 0.5

        h_std = np.std(h[mask])
        s_std = np.std(s[mask])

        # Normalize and combine
        # Lower std indicates more uniform color
        h_uniformity = max(0, 1 - h_std / 180.0)
        s_uniformity = max(0, 1 - s_std / 255.0)

        return (h_uniformity + s_uniformity) / 2

    except Exception:
        return 0.5


def analyze_surface_defects(image: np.ndarray) -> List[Dict[str, float]]:
    """
    Analyze surface quality and detect defects.

    Args:
        image: Fruit image region

    Returns:
        List of detected defects with bounding boxes and severity scores.
    """
    defects: List[Dict[str, float]] = []

    try:
        gray = cv2.cvtColor(image, cv2.COLOR_BGR2GRAY)
        blurred = cv2.GaussianBlur(gray, (5, 5), 0)
        edges = cv2.Canny(blurred, 40, 120)

        kernel = cv2.getStructuringElement(cv2.MORPH_ELLIPSE, (3, 3))
        dilated = cv2.dilate(edges, kernel, iterations=1)
        contours, _ = cv2.findContours(dilated, cv2.RETR_EXTERNAL, cv2.CHAIN_APPROX_SIMPLE)

        for contour in contours:
            area = cv2.contourArea(contour)
            if area < 20:
                continue

            x, y, w, h = cv2.boundingRect(contour)
            bbox_area = float(max(w * h, 1))
            severity = float(np.clip(area / bbox_area, 0.0, 1.0))

            defects.append(
                {
                    "bbox": [int(x), int(y), int(w), int(h)],
                    "area": float(area),
                    "severity": severity,
                }
            )

    except Exception as e:
        logger.warning("Surface defect analysis failed: %s", str(e))
        return defects

    defects.sort(key=lambda item: item["area"], reverse=True)
    return defects


def _estimate_surface_quality(defects: List[Dict[str, float]], shape: Tuple[int, int]) -> float:
    """Convert defect detections into a normalized surface quality score."""
    try:
        height, width = shape
        image_area = max(height * width, 1)

        if not defects:
            return 1.0

        total_defect_area = sum(defect["area"] for defect in defects)
        mean_severity = float(np.mean([defect["severity"] for defect in defects])) if defects else 0.0

        coverage = float(np.clip(total_defect_area / image_area, 0.0, 1.0))
        score = 1.0 - (coverage * 0.7 + mean_severity * 0.3)
        return float(np.clip(score, 0.0, 1.0))
    except Exception:
        return 0.5


def calculate_size_score(shape: Tuple[int, int]) -> float:
    """
    Calculate size score based on fruit dimensions.
    This function adapts optimal size ranges based on fruit type if available.

    Args:
        shape: Fruit region shape (height, width)

    Returns:
        Size score (0-1)
    """
    try:
        height, width = shape
        area = height * width

        # Default optimal size range
        min_area = 1000   # Minimum acceptable size
        optimal_area = 5000  # Optimal size
        max_area = 15000  # Maximum before considering too large

        # Normalize score based on area
        if area < min_area:
            return 0.3
        elif area <= optimal_area:
            return 0.5 + 0.5 * (area - min_area) / (optimal_area - min_area)
        elif area <= max_area:
            return 1.0 - 0.3 * (area - optimal_area) / (max_area - optimal_area)
        else:
            # Size is too large, but not as penalized as too small
            return max(0.2, 0.4 - 0.2 * (area - max_area) / max_area)

    except Exception:
        return 0.5


def analyze_freshness(lab_image: np.ndarray, fruit_class: Optional[str] = None) -> float:
    """
    Analyze fruit freshness based on color characteristics.
    Takes into account fruit-specific color patterns.

    Args:
        lab_image: Image in LAB color space
        fruit_class: Optional fruit class name for class-specific analysis

    Returns:
        Freshness score (0-1)
    """
    try:
        l, a, b = cv2.split(lab_image)

        # Calculate mean brightness (L channel)
        mean_brightness = np.mean(l)

        # Calculate color vibrancy (a and b channels)
        color_variance = np.var(a) + np.var(b)

        # Class-specific freshness analysis
        if fruit_class:
            # Different fruits have different freshness indicators
            if 'apple' in fruit_class.lower():
                # Apples: look for skin texture and color consistency
                freshness_score = analyze_apple_freshness(l, a, b)
            elif 'banana' in fruit_class.lower():
                # Bananas: look for brown spots and yellow color intensity
                freshness_score = analyze_banana_freshness(lab_image)
            elif 'orange' in fruit_class.lower() or 'citrus' in fruit_class.lower():
                # Citrus: look for color and surface texture
                freshness_score = analyze_citrus_freshness(l, a, b)
            else:
                # General calculation for other fruits
                brightness_score = min(1.0, mean_brightness / 200.0)
                vibrancy_score = min(1.0, color_variance / 1000.0)
                freshness_score = (brightness_score + vibrancy_score) / 2
        else:
            # General calculation when no class is specified
            brightness_score = min(1.0, mean_brightness / 200.0)
            vibrancy_score = min(1.0, color_variance / 1000.0)
            freshness_score = (brightness_score + vibrancy_score) / 2

        return freshness_score

    except Exception:
        return 0.5


def analyze_apple_freshness(l: np.ndarray, a: np.ndarray, b: np.ndarray) -> float:
    """Analyze freshness specifically for apples."""
    # Apples should have consistent color and brightness
    brightness_score = min(1.0, np.mean(l) / 200.0)
    color_consistency = 1.0 - (np.std(a) / 255.0 + np.std(b) / 255.0) / 2
    
    return (brightness_score + color_consistency) / 2


def analyze_banana_freshness(lab_image: np.ndarray) -> float:
    """Analyze freshness specifically for bananas."""
    # For bananas, look for brown spots which indicate overripeness
    l, a, b = cv2.split(lab_image)
    
    # Look for brown spots (high a and low b values)
    brown_mask = (a > 150) & (b < 120)
    brown_ratio = np.sum(brown_mask) / np.prod(l.shape)
    
    # Calculate ripeness based on yellow color (b channel dominance)
    yellow_score = np.mean(b) / 255.0
    brown_penalty = min(0.5, brown_ratio * 0.5)
    
    ripeness_score = max(0.3, yellow_score - brown_penalty)
    return ripeness_score


def analyze_citrus_freshness(l: np.ndarray, a: np.ndarray, b: np.ndarray) -> float:
    """Analyze freshness specifically for citrus fruits."""
    # Citrus should have vibrant color and good skin texture
    brightness_score = min(1.0, np.mean(l) / 200.0)
    color_vibrancy = (np.mean(b) / 255.0 + (1.0 - np.std(a) / 255.0)) / 2
    
    return (brightness_score + color_vibrancy) / 2


def filter_overlapping_detections(detections: List[Dict], iou_threshold: float = 0.5) -> List[Dict]:
    """
    Filter overlapping detections using Non-Maximum Suppression.

    Args:
        detections: List of detection dictionaries
        iou_threshold: IoU threshold for suppression

    Returns:
        Filtered detections
    """
    if not detections:
        return []

    # Sort by confidence
    detections = sorted(detections, key=lambda x: x['confidence'], reverse=True)

    filtered = []
    used_indices = set()

    for i, det1 in enumerate(detections):
        if i in used_indices:
            continue

        filtered.append(det1)

        for j, det2 in enumerate(detections[i + 1:], i + 1):
            if j in used_indices:
                continue

            # Calculate IoU
            iou = calculate_iou(det1['bbox'], det2['bbox'])

            if iou > iou_threshold:
                used_indices.add(j)

    return filtered


def calculate_iou(box1: List[float], box2: List[float]) -> float:
    """
    Calculate Intersection over Union (IoU) of two bounding boxes.

    Args:
        box1: First bounding box [x1, y1, x2, y2]
        box2: Second bounding box [x1, y1, x2, y2]

    Returns:
        IoU score
    """
    x1 = max(box1[0], box2[0])
    y1 = max(box1[1], box2[1])
    x2 = min(box1[2], box2[2])
    y2 = min(box1[3], box2[3])

    if x2 <= x1 or y2 <= y1:
        return 0.0

    intersection = (x2 - x1) * (y2 - y1)
    area1 = (box1[2] - box1[0]) * (box1[3] - box1[1])
    area2 = (box2[2] - box2[0]) * (box2[3] - box2[1])
    union = area1 + area2 - intersection

    return intersection / union if union > 0 else 0.0
