# pyxclip

Cross-platform clipboard access with zero external dependencies. No xclip or xsel needed on Linux. Powered by Rust (PyO3 + arboard).

## Installation

```bash
pip install pyxclip
```

Requires Python 3.8 or newer. No C compiler, no system packages.

## Quick Start

```python
import pyxclip

pyxclip.copy("Hello, world!")
print(pyxclip.paste())   # "Hello, world!"
pyxclip.clear()
```

## API Reference

### `copy(data)`

Polymorphic. Copies text or images depending on the argument type.

```python
# Copy text
pyxclip.copy("some text")

# Copy an image via (width, height, rgba_bytes) tuple
width, height = 4, 4
pixels = b"\xff\x00\x00\xff" * (width * height)  # red, fully opaque
pyxclip.copy((width, height, pixels))
```

Passing any other type (e.g. `int`, `list`) raises `TypeError`.

### `copy_image(width, height, rgba_bytes)`

Explicit alternative for copying images. `rgba_bytes` is raw RGBA pixel data, 4 bytes per pixel, row-major order.

```python
# 2x2 red square
pixels = bytes([255, 0, 0, 255] * 4)
pyxclip.copy_image(2, 2, pixels)
```

### `copy_files(paths)`

Copy file paths to the clipboard. Accepts a list of strings or path-like objects.

```python
pyxclip.copy_files(["/tmp/report.pdf", "/home/user/image.png"])
```

Not supported on all platforms. Raises `ClipboardError` when the platform does not support it.

### `paste() -> str | dict`

Returns clipboard content. Tries text first; if the clipboard holds an image, returns a dict with keys `"width"`, `"height"`, and `"bytes"`.

```python
# Text
text = pyxclip.paste()          # returns str

# Image
result = pyxclip.paste()        # returns dict
# result = {"width": 800, "height": 600, "bytes": b"\xff\x00..."}
```

Raises `ClipboardError` when the clipboard is empty or contains incompatible data.

### `clear()`

Empties the clipboard.

```python
pyxclip.clear()
```

### `ClipboardError`

Subclass of `RuntimeError`. Raised on all clipboard failures with a descriptive message.

```python
import pyxclip

try:
    pyxclip.paste()
except pyxclip.ClipboardError as e:
    print(f"Clipboard failed: {e}")
```

Common messages:

- `"No display server available (headless?)"` — no `DISPLAY` or `WAYLAND_DISPLAY` set
- `"Clipboard is empty or contains incompatible data"` — nothing to paste
- `"Clipboard is held by another process; retry later"` — transient lock contention

### `__version__`

```python
print(pyxclip.__version__)
```

## Error Handling

```python
import pyxclip

# Headless CI detection
import os
if not os.environ.get("DISPLAY") and not os.environ.get("WAYLAND_DISPLAY"):
    print("Skipping clipboard test: no display server")
else:
    pyxclip.copy("works!")
    assert pyxclip.paste() == "works!"

# Guarding with try/except
try:
    pyxclip.paste()
except pyxclip.ClipboardError:
    print("Nothing on the clipboard right now")
```

## Platform Support

| Platform | Backend | Notes |
|---|---|---|
| Linux | X11 or Wayland | No xclip/xsel. Wayland uses `wayland-data-control`. |
| macOS | NSPasteboard | System framework, no extras needed. |
| Windows | Win32 clipboard | System API, no extras needed. |

On headless Linux (no `DISPLAY`, no `WAYLAND_DISPLAY`), all operations raise `ClipboardError`.

## License

MIT
