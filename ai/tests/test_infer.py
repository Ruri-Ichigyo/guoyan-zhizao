import pytest
import numpy as np
import torch
from unittest.mock import Mock, patch

import sys
import os
sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from main import FruitDetectionService
from utils.preprocess import preprocess_image, letterbox
from utils.postprocess import calculate_quality_grade, calculate_iou

class TestFruitDetectionService:

    @pytest.fixture
    def service(self):
        mock_model = Mock()
        mock_classifier = Mock()
        mock_classifier.in_features = 1024
        mock_model.classifier = mock_classifier
        mock_model.to.return_value = mock_model
        mock_model.eval.return_value = mock_model
        mock_model.load_state_dict.return_value = None

        with patch('main.models.densenet121', return_value=mock_model), \
             patch('main.boto3.client'), \
             patch('main.KafkaConsumer'), \
             patch('main.KafkaProducer'):
            service_instance = FruitDetectionService()
            mock_logits = torch.randn(1, service_instance.num_classes)
            service_instance.model.return_value = mock_logits
            return service_instance

    def test_load_labels(self, service):
        """Test label loading."""
        assert isinstance(service.labels, dict)
        assert len(service.labels) > 0

    def test_preprocess_image(self):
        """Test image preprocessing."""
        # Create test image
        test_image = np.random.randint(0, 255, (480, 640, 3), dtype=np.uint8)

        # Preprocess
        processed = preprocess_image(test_image)

        # Check output properties
        assert isinstance(processed, torch.Tensor)
        assert processed.shape == (3, 224, 224)
        assert processed.dtype == torch.float32
        assert torch.isfinite(processed).all()

    def test_letterbox_resize(self):
        """Test letterbox resizing."""
        test_image = np.random.randint(0, 255, (300, 400, 3), dtype=np.uint8)

        resized = letterbox(test_image, (640, 640))

        assert resized.shape == (640, 640, 3)

    def test_quality_grade_calculation(self):
        """Test quality grade calculation."""
        # Create test fruit region
        fruit_region = np.random.randint(0, 255, (100, 100, 3), dtype=np.uint8)

        grade = calculate_quality_grade(fruit_region, "apple", 0.95)

        assert grade in ['A', 'B', 'C']

    def test_iou_calculation(self):
        """Test IoU calculation."""
        box1 = [10, 10, 50, 50]  # 40x40 box
        box2 = [30, 30, 70, 70]  # 40x40 box, overlapping

        iou = calculate_iou(box1, box2)

        # Expected IoU for these boxes
        intersection = 20 * 20  # 400
        union = 40 * 40 + 40 * 40 - 400  # 2800
        expected_iou = 400 / 2800

        assert abs(iou - expected_iou) < 0.01

    def test_empty_fruit_region_quality(self):
        """Test quality grade for empty region."""
        empty_region = np.array([])

        grade = calculate_quality_grade(empty_region, "apple", 0.8)

        assert grade == 'C'

    @patch('main.FruitDetectionService.download_image_from_s3')
    @patch('main.FruitDetectionService.detect_fruits')
    @patch('main.FruitDetectionService.send_result_to_kafka')
    def test_process_message(self, mock_kafka, mock_detect, mock_download, service):
        """Test message processing."""
        # Mock return values
        mock_download.return_value = np.random.randint(0, 255, (480, 640, 3), dtype=np.uint8)
        mock_detect.return_value = [
            {
                'class_name': 'apple',
                'confidence': 0.95,
                'bbox': [10, 10, 50, 50],
                'quality_grade': 'A'
            }
        ]

        # Test message
        test_message = {
            'upload_id': 'test-123',
            'object_key': 'test/image.jpg'
        }

        # Process message
        import asyncio
        asyncio.run(service.process_message(test_message))

        # Verify calls
        mock_download.assert_called_once_with('test/image.jpg')
        mock_detect.assert_called_once()
        mock_kafka.assert_called_once()

if __name__ == "__main__":
    pytest.main([__file__])
