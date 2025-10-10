"""
Minimal stub implementation of the ``torchvision`` package used in tests.

Only the APIs exercised by the unit tests are implemented. When the real
library is available it should be preferred, but in constrained environments
this lightweight version keeps the application testable.
"""

from . import models, transforms

__all__ = ["models", "transforms"]
