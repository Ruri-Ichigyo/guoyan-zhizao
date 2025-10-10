"""
Subset of torchvision transforms required by the tests.
"""

from __future__ import annotations

from typing import Callable, Iterable, Tuple

import numpy as np
from PIL import Image

import torch


class Compose:
    def __init__(self, transforms: Iterable[Callable]) -> None:
        self.transforms = list(transforms)

    def __call__(self, x):
        for transform in self.transforms:
            x = transform(x)
        return x


class Resize:
    def __init__(self, size: Tuple[int, int]) -> None:
        self.size = size

    def __call__(self, image: Image.Image) -> Image.Image:
        return image.resize(self.size, Image.BILINEAR)


class ToTensor:
    def __call__(self, image: Image.Image) -> torch.Tensor:
        array = np.asarray(image, dtype=np.float32) / 255.0
        if array.ndim == 2:
            array = np.expand_dims(array, axis=-1)
        array = np.transpose(array, (2, 0, 1))
        return torch.tensor(array)


class Normalize:
    def __init__(self, mean, std) -> None:
        self.mean = np.asarray(mean, dtype=np.float32)[:, None, None]
        self.std = np.asarray(std, dtype=np.float32)[:, None, None]

    def __call__(self, tensor: torch.Tensor) -> torch.Tensor:
        data = tensor.numpy()
        normalized = (data - self.mean) / np.clip(self.std, a_min=1e-12, a_max=None)
        return torch.tensor(normalized)
