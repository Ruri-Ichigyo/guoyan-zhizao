"""
Stub implementations of torchvision models required for the tests.
"""

from __future__ import annotations

from types import SimpleNamespace
from typing import Optional

import numpy as np

import torch


class DenseNet121_Weights:
    IMAGENET1K_V1 = "IMAGENET1K_V1"


class _DummyModel:
    def __init__(self, feature_dim: int = 1024) -> None:
        self.classifier = SimpleNamespace(in_features=feature_dim)

    def to(self, *_args, **_kwargs) -> "_DummyModel":
        return self

    def eval(self) -> "_DummyModel":
        return self

    def __call__(self, inputs: torch.Tensor) -> torch.Tensor:
        data = np.asarray(inputs, dtype=np.float32)
        batch = 1 if data.ndim < 2 else data.shape[0]
        return torch.tensor(np.zeros((batch, self.classifier.in_features), dtype=np.float32))


def densenet121(weights: Optional[DenseNet121_Weights] = None) -> _DummyModel:
    # The weights argument is ignored in the stub but retained for API parity.
    _ = weights
    return _DummyModel()
