"""Cross-platform clipboard access with zero external dependencies."""

from __future__ import annotations

from ._core import (
    ClipboardError,
    __version__,
    clear,
    copy,
    copy_files,
    copy_image,
    paste,
)

__all__ = [
    "__version__",
    "ClipboardError",
    "clear",
    "copy",
    "copy_files",
    "copy_image",
    "paste",
]
