"""
Minimal stub implementation of a subset of the PyTorch API used in tests.

This stub is activated when the real `torch` package is unavailable. It provides
just enough tensor functionality to support the unit tests, without depending on
native extensions or large external wheels.
"""

from __future__ import annotations

import sys
import types
from contextlib import ContextDecorator
from dataclasses import dataclass
from typing import Iterator, Sequence, Tuple, Union

import numpy as np

Number = Union[int, float]
ArrayLike = Union["Tensor", Sequence[Number], np.ndarray]

__all__ = [
    "Tensor",
    "tensor",
    "linspace",
    "randn",
    "softmax",
    "topk",
    "no_grad",
    "device",
    "cuda",
    "nn",
    "float32",
]


def _to_numpy(data: ArrayLike) -> np.ndarray:
    """Convert supported inputs to a NumPy array."""
    if isinstance(data, Tensor):
        return data._data
    return np.asarray(data, dtype=np.float32)


class Tensor:
    """Very small tensor abstraction backed by NumPy."""

    __slots__ = ("_data",)

    def __init__(self, data: ArrayLike) -> None:
        self._data = _to_numpy(data).astype(np.float32, copy=True)

    def unsqueeze(self, dim: int) -> "Tensor":
        return Tensor(np.expand_dims(self._data, axis=dim))

    def float(self) -> "Tensor":
        return Tensor(self._data.astype(np.float32, copy=True))

    def to(self, *_args, **_kwargs) -> "Tensor":
        # Device handling is ignored in the stub.
        return self

    def detach(self) -> "Tensor":
        return Tensor(self._data)

    def numpy(self) -> np.ndarray:
        return self._data.copy()

    def item(self) -> float:
        return float(self._data.item())

    @property
    def shape(self) -> Tuple[int, ...]:
        return self._data.shape

    def __iter__(self) -> Iterator["Tensor"]:
        for item in self._data:
            yield Tensor(item)

    def __getitem__(self, index) -> "Tensor":
        return Tensor(self._data[index])

    def __array__(self, dtype=None) -> np.ndarray:
        if dtype is None:
            return self._data.copy()
        return self._data.astype(dtype, copy=True)

    def __matmul__(self, other: ArrayLike) -> "Tensor":
        return Tensor(self._data @ _to_numpy(other))

    def __add__(self, other: ArrayLike) -> "Tensor":
        return Tensor(self._data + _to_numpy(other))

    def topk(self, k: int, dim: int = -1) -> Tuple["Tensor", "Tensor"]:
        return topk(self, k, dim=dim)

    def __repr__(self) -> str:  # pragma: no cover - debugging helper
        return f"Tensor({self._data!r})"


float32 = np.float32


def tensor(data: ArrayLike, dtype=None) -> Tensor:
    arr = _to_numpy(data)
    if dtype is not None:
        arr = arr.astype(dtype, copy=False)
    return Tensor(arr)


def linspace(start: Number, end: Number, steps: int) -> Tensor:
    return Tensor(np.linspace(start, end, num=steps, dtype=np.float32))


def randn(*shape: int) -> Tensor:
    return Tensor(np.random.normal(loc=0.0, scale=1.0, size=shape).astype(np.float32))


def softmax(input_tensor: Tensor, dim: int = -1) -> Tensor:
    data = _to_numpy(input_tensor)
    if data.size == 0:
        return Tensor(data)

    axis = dim if dim >= 0 else data.ndim + dim
    shifted = data - np.max(data, axis=axis, keepdims=True)
    exp = np.exp(shifted)
    total = np.sum(exp, axis=axis, keepdims=True)
    return Tensor(exp / np.clip(total, a_min=1e-12, a_max=None))


def topk(input_tensor: Tensor, k: int, dim: int = -1) -> Tuple[Tensor, Tensor]:
    data = _to_numpy(input_tensor)
    axis = dim if dim >= 0 else data.ndim + dim

    # Partition to obtain top-k indices, then sort them by value.
    partition_indices = np.argpartition(-data, kth=k - 1, axis=axis)
    top_indices = np.take(partition_indices, np.arange(k), axis=axis)
    top_values = np.take_along_axis(data, top_indices, axis=axis)

    order = np.argsort(-top_values, axis=axis)
    sorted_indices = np.take_along_axis(top_indices, order, axis=axis)
    sorted_values = np.take_along_axis(top_values, order, axis=axis)

    return Tensor(sorted_values), Tensor(sorted_indices.astype(np.int64))


class _NoGrad(ContextDecorator):
    def __enter__(self):
        return None

    def __exit__(self, exc_type, exc, tb):
        return False


def no_grad() -> _NoGrad:
    return _NoGrad()


@dataclass
class device:
    """Lightweight device representation."""

    type: str

    def __str__(self) -> str:
        return self.type


class cuda:
    @staticmethod
    def is_available() -> bool:
        return False


class _Module:
    """Minimal stand-in for torch.nn.Module."""

    def __init__(self) -> None:
        self.training = False

    def to(self, *_args, **_kwargs):
        return self

    def eval(self):
        self.training = False
        return self

    def state_dict(self):
        return {}

    def load_state_dict(self, _state):
        return None


class _Linear(_Module):
    def __init__(self, in_features: int, out_features: int) -> None:
        super().__init__()
        self.in_features = in_features
        self.out_features = out_features
        self.weight = np.zeros((out_features, in_features), dtype=np.float32)
        self.bias = np.zeros(out_features, dtype=np.float32)

    def __call__(self, inputs: Tensor) -> Tensor:
        data = _to_numpy(inputs)
        result = data @ self.weight.T + self.bias
        return Tensor(result)


nn = types.ModuleType("torch.nn")
nn.Linear = _Linear
nn.Module = _Module
sys.modules[__name__ + ".nn"] = nn
