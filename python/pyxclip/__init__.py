"""Cross-platform clipboard access with zero external dependencies."""

from __future__ import annotations

from ._core import (
    ClipboardError,
    __version__,
    copy,
    paste,
)

__all__ = [
    "__version__",
    "ClipboardError",
    "copy",
    "paste",
]
