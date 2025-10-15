import os
import sys
from unittest.mock import Mock, patch

import cv2
import numpy as np
import pytest
import torch

# 添加项目路径
sys.path.append(os.path.join(os.path.dirname(__file__), '../../ai'))

from main import FruitDetectionService  # noqa: E402
from utils.preprocess import letterbox, preprocess_image  # noqa: E402
from utils.postprocess import (  # noqa: E402
    calculate_color_uniformity,
    calculate_iou,
    calculate_quality_grade,
    calculate_size_score,
    analyze_surface_defects,
)


class TestFruitDetectionService:
    """针对当前 FastAPI 推理服务的核心逻辑进行单元测试"""

    @pytest.fixture
    def service(self):
        mock_model = Mock()
        mock_classifier = Mock()
        mock_classifier.in_features = 1024
        mock_model.classifier = mock_classifier
        mock_model.to.return_value = mock_model
        mock_model.eval.return_value = mock_model

        with patch('main.models.densenet121', return_value=mock_model), \
             patch('main.os.path.exists', return_value=False):
            service_instance = FruitDetectionService()
            mock_logits = torch.linspace(1, service_instance.num_classes, steps=service_instance.num_classes)
            mock_logits = mock_logits.unsqueeze(0).float()
            service_instance.model.return_value = mock_logits
            return service_instance

    @pytest.fixture
    def test_image(self):
        return np.random.randint(0, 255, (480, 640, 3), dtype=np.uint8)

    def test_load_labels(self, service):
        assert isinstance(service.labels, dict)
        assert service.num_classes == len(service.labels)
        assert service.top_k <= service.num_classes

    def test_decode_invalid_image(self, service):
        with pytest.raises(ValueError):
            service._decode_image(b'invalid-bytes')

    def test_infer_with_valid_image(self, service):
        image = np.full((224, 224, 3), 120, dtype=np.uint8)
        success, buffer = cv2.imencode('.jpg', image)
        assert success

        with patch('main.calculate_quality_grade', return_value='A'):
            response = service.infer('test-upload', buffer.tobytes())

        assert response.status == 'completed'
        assert response.total_fruits == service.top_k
        assert len(response.results) == service.top_k
        assert all(result.quality_grade in {'A', 'B', 'C'} for result in response.results)

    def test_infer_rejects_empty_bytes(self, service):
        with pytest.raises(ValueError):
            service.infer('test-upload', b'')


class TestImageProcessingUtils:
    def test_preprocess_image(self, test_image):
        processed = preprocess_image(test_image)
        assert isinstance(processed, torch.Tensor)
        assert processed.shape == (3, 224, 224)

    def test_letterbox_resize(self, test_image):
        resized = letterbox(test_image, (640, 640))
        assert resized.shape == (640, 640, 3)

    def test_quality_grade_calculation(self):
        high_quality = np.full((100, 100, 3), [200, 120, 80], dtype=np.uint8)
        low_quality = np.random.randint(0, 255, (100, 100, 3), dtype=np.uint8)

        high_grade = calculate_quality_grade(high_quality, 'apple', 0.95)
        low_grade = calculate_quality_grade(low_quality, 'apple', 0.5)

        grade_order = {'A': 3, 'B': 2, 'C': 1}
        assert grade_order[high_grade] >= grade_order[low_grade]

    def test_iou_calculation(self):
        box1 = [10, 10, 50, 50]
        box2 = [30, 30, 70, 70]
        iou = calculate_iou(box1, box2)
        assert 0 <= iou <= 1

    def test_color_uniformity(self):
        rgb_image = np.full((80, 80, 3), [120, 140, 90], dtype=np.uint8)
        hsv_image = cv2.cvtColor(rgb_image, cv2.COLOR_RGB2HSV)
        score = calculate_color_uniformity(hsv_image)
        assert 0 <= score <= 1

    def test_surface_defect_analysis(self):
        clean_image = np.full((200, 200, 3), [150, 100, 80], dtype=np.uint8)
        defects_clean = analyze_surface_defects(clean_image)

        defective_image = clean_image.copy()
        cv2.line(defective_image, (50, 50), (150, 50), (0, 0, 0), 3)
        defects_dirty = analyze_surface_defects(defective_image)

        assert len(defects_dirty) >= len(defects_clean)

    def test_size_score(self):
        assert 0 <= calculate_size_score((50, 50)) <= 1
        assert 0 <= calculate_size_score((120, 120)) <= 1
        assert 0 <= calculate_size_score((300, 300)) <= 1
