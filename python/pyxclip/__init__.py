"""Cross-platform clipboard access with zero external dependencies."""

from __future__ import annotations

from ._core import ClipboardError, __version__, clear_clipboard, copy_text, paste_text

__all__ = [
    "__version__",
    "ClipboardError",
    "clear_clipboard",
    "copy_text",
    "paste_text",
]
