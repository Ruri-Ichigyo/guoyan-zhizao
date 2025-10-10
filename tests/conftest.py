import numpy as np
import pytest


@pytest.fixture
def test_image():
    """Provide a reusable pseudo-random test image."""
    rng = np.random.default_rng(seed=1234)
    return rng.integers(0, 255, size=(480, 640, 3), dtype=np.uint8)
