import json
import os
import sys
from pathlib import Path
from unittest.mock import Mock, patch

import cv2
import numpy as np
import pytest
import torch

sys.path.append(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))

from main import FruitDetectionService
from utils.preprocess import letterbox, preprocess_image
from utils.postprocess import calculate_iou, calculate_quality_grade


@pytest.fixture
def setup_training_dir(tmp_path, monkeypatch):
    training_dir = tmp_path / 'Training'
    training_dir.mkdir()
    for name in ['Apple', 'Banana', 'Cherry']:
        (training_dir / name).mkdir()

    monkeypatch.chdir(tmp_path)
    monkeypatch.setenv('TRAINING_DATA_DIR', str(training_dir))
    return training_dir


@pytest.fixture
def service(setup_training_dir):
    mock_model = Mock()
    mock_classifier = Mock()
    mock_classifier.in_features = 1024
    mock_model.classifier = mock_classifier
    mock_model.to.return_value = mock_model
    mock_model.eval.return_value = mock_model

    with patch('main.models.densenet121', return_value=mock_model):
        svc = FruitDetectionService()

    svc.model = Mock(return_value=torch.randn(1, svc.num_classes))
    return svc


def test_labels_synced_from_training(service, setup_training_dir):
    labels_path = Path('model/labels.json')
    assert labels_path.exists()

    with labels_path.open('r', encoding='utf-8') as file:
        data = json.load(file)

    expected = {
        str(index): name for index, name in enumerate(sorted(os.listdir(setup_training_dir)))
    }
    assert data == expected
    assert service.labels == {int(k): v for k, v in expected.items()}


def test_infer_generates_results(service, tmp_path):
    image = np.full((224, 224, 3), 128, dtype=np.uint8)
    success, buffer = cv2.imencode('.jpg', image)
    assert success

    with patch('main.calculate_quality_grade', return_value='A'):
        response = service.infer('upload-1', buffer.tobytes())

    assert response.status == 'completed'
    assert response.total_fruits == service.top_k
    assert all(result.quality_grade == 'A' for result in response.results)


def test_preprocess_pipeline():
    test_image = np.random.randint(0, 255, (480, 640, 3), dtype=np.uint8)
    processed = preprocess_image(test_image)
    assert isinstance(processed, torch.Tensor)
    assert processed.shape == (3, 224, 224)


def test_letterbox_resize():
    test_image = np.random.randint(0, 255, (300, 400, 3), dtype=np.uint8)
    resized = letterbox(test_image, (640, 640))
    assert resized.shape == (640, 640, 3)


def test_quality_grade_and_iou():
    fruit_region = np.random.randint(0, 255, (100, 100, 3), dtype=np.uint8)
    grade = calculate_quality_grade(fruit_region, 'apple', 0.9)
    assert grade in {'A', 'B', 'C'}

    box1 = [10, 10, 50, 50]
    box2 = [30, 30, 70, 70]
    iou = calculate_iou(box1, box2)
    assert 0 <= iou <= 1
