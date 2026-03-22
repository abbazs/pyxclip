# pyxclip

[![PyPI version](https://img.shields.io/pypi/v/pyxclip)](https://pypi.org/project/pyxclip/)
[![Python versions](https://img.shields.io/pypi/pyversions/pyxclip)](https://pypi.org/project/pyxclip/)
[![Downloads](https://img.shields.io/pypi/dm/pyxclip)](https://pypi.org/project/pyxclip/)

Cross-platform clipboard access with zero external dependencies. No xclip or xsel needed on Linux. Built with Rust (PyO3 + arboard) for speed and reliability.

## Installation

```bash
pip install pyxclip
```

Works with Python 3.8 through 3.14. No C compiler or system packages required.

## Quick Start

```python
import pyxclip

pyxclip.copy("Hello, world!")
text = pyxclip.paste()
print(text)  # Hello, world!
```

That's the whole API. Three functions: `copy()`, `paste()`, and `clear()`. Copy figures out what to do based on the type you pass. Paste returns whatever is on the clipboard.

## Text Operations

Copy and paste strings, unicode, multiline text, even empty strings.

```python
import pyxclip

# Basic text
pyxclip.copy("Hello, world!")
print(pyxclip.paste())  # Hello, world!

# Unicode and emoji
pyxclip.copy("cafe\u0301, na\u00efvet\u00e9, \U0001f600")
print(pyxclip.paste())  # cafe, naiveté, 😀

# Multiline text
blocks = """First line
Second line
Third line"""
pyxclip.copy(blocks)
print(pyxclip.paste())
# First line
# Second line
# Third line

# Empty string clears to empty text
pyxclip.copy("")
print(pyxclip.paste())  # (empty string)
```

## Clear the Clipboard

Pass `None` to `copy()` or call `clear()`.

```python
import pyxclip

# Either way works
pyxclip.copy(None)
pyxclip.clear()

# Pasting after clearing raises ClipboardError
try:
    pyxclip.paste()
except pyxclip.ClipboardError:
    print("Clipboard is empty")
```

## Image Operations

Copy images by passing a `(width, height, pixels)` tuple where `pixels` is raw RGBA data, 4 bytes per pixel, row-major order. Paste returns a dict with `"width"`, `"height"`, and `"bytes"` keys.
```python
import pyxclip

# Create a 4x4 red square
width, height = 4, 4
pixels = b"\xff\x00\x00\xff" * (width * height)  # RGBA: red, fully opaque
pyxclip.copy((width, height, pixels))

# Paste the image back
result = pyxclip.paste()
print(type(result))        # <class 'dict'>
print(result["width"])     # 4
print(result["height"])    # 4
print(len(result["bytes"]))  # 64 (4 * 4 * 4 bytes)

# Save pasted image to a PNG file
from PIL import Image

result = pyxclip.paste()
img = Image.frombytes("RGBA", (result["width"], result["height"]), result["bytes"])
img.save("screenshot.png")
```

## File Operations

Copy file paths to the clipboard as a single `Path` or a list. Paste returns a list of strings.

```python
import pyxclip
from pathlib import Path

# Single file
pyxclip.copy(Path("/tmp/report.pdf"))

# Multiple files
files = [Path("/tmp/report.pdf"), Path.home() / "photos" / "vacation.jpg"]
pyxclip.copy(files)

# Paste returns a list
pasted = pyxclip.paste()
print(type(pasted))  # <class 'list'>
print(pasted)        # ['/tmp/report.pdf', '/home/user/photos/vacation.jpg']
```

### str vs Path — why it matters

`str` is always treated as **text**, never as a file path. This is intentional:

```python
import pyxclip
from pathlib import Path

pyxclip.copy("/some/path.txt")
print(pyxclip.paste())  # "/some/path.txt" — copied as text

pyxclip.copy(Path("/some/path.txt"))
# — copied as file reference (if file exists)
```

If you have a string that represents a file path, wrap it in `Path`:

```python
pyxclip.copy(Path(my_string_path))
```

### Error messages

File copy errors are specific about what went wrong:

```python
import pyxclip
from pathlib import Path

try:
    pyxclip.copy(Path("/does/not/exist.txt"))
except pyxclip.ClipboardError as e:
    print(e)
    # "Failed to copy file paths to clipboard: one or more files may not exist
    #  or are inaccessible. All paths must point to existing files or directories."
```

File path copy succeeds on all desktop platforms (Windows, macOS, Linux). However, **pasting files into a file manager depends on your desktop environment**. pyxclip writes the standard `text/uri-list` format — whether the receiving app reads it is up to that app.

If `copy()` succeeds but pasting into a specific app doesn't work, the issue is with that app's clipboard support — not pyxclip.

## Error Handling

All clipboard failures raise `ClipboardError`, a subclass of `RuntimeError`.

```python
import pyxclip

# Catch any clipboard failure
try:
    pyxclip.paste()
except pyxclip.ClipboardError as e:
    print(f"Failed: {e}")

# Detect headless environments before trying
import os

def clipboard_available() -> bool:
    if os.environ.get("DISPLAY") or os.environ.get("WAYLAND_DISPLAY"):
        return True
    if os.name == "nt" or sys.platform == "darwin":
        return True
    return False

if clipboard_available():
    pyxclip.copy("works!")
else:
    print("No clipboard in this environment")
```

Common errors:

- `"No display server available (headless?)"` -- no display set on Linux
- `"Clipboard is empty or contains incompatible data"` -- nothing readable
- `"Clipboard is held by another process; retry later"` -- transient lock

## Practical Examples

### Copy the result of a script to clipboard

```python
import pyxclip
import json

data = {"status": "ok", "items": [1, 2, 3]}
pyxclip.copy(json.dumps(data, indent=2))
# Now Ctrl+V in any editor pastes formatted JSON
```

### Pipe data through the clipboard in shell scripts

```bash
# Sort lines from clipboard and put them back
python3 -c "
import pyxclip
lines = pyxclip.paste().strip().split('\n')
pyxclip.copy('\n'.join(sorted(set(lines))))
"

# Reverse the clipboard content
python3 -c "
import pyxclip
t = pyxclip.paste()
pyxclip.copy(t[::-1])
"
```

### Clipboard-based communication between scripts

```python
# producer.py
import pyxclip
import time

for i in range(5):
    pyxclip.copy(f"task-{i}")
    time.sleep(1)

# consumer.py
import pyxclip

last = None
while True:
    current = pyxclip.paste()
    if current != last:
        print(f"New item: {current}")
        last = current
```

### Paste clipboard content into a file
```python
import pyxclip

content = pyxclip.paste()
with open("clipboard_dump.txt", "w") as f:
    f.write(content)
print(f"Saved {len(content)} characters")
```

### Copy an API response to clipboard

```python
import pyxclip
import urllib.request

resp = urllib.request.urlopen("https://httpbin.org/get")
data = resp.read().decode()
pyxclip.copy(data)
print("API response copied to clipboard")
```

### Quick notes manager

```python
import pyxclip
from datetime import datetime

# Append a timestamped note
note = pyxclip.paste()
with open("notes.txt", "a") as f:
    f.write(f"[{datetime.now():%Y-%m-%d %H:%M}] {note}\n")
print("Note saved")
```

### Copy a URL from a script

```python
import pyxclip

url = "https://github.com/abbazs/pyxclip"
pyxclip.copy(url)
print(f"Copied: {url} -- paste in browser")
```

### Copy source code to clipboard

```python
import pyxclip

code = '''
def fibonacci(n):
    a, b = 0, 1
    for _ in range(n):
        a, b = b, a + b
    return a
'''
pyxclip.copy(code.strip())
```

### Grab and re-copy clipboard text with modifications

```python
import pyxclip

text = pyxclip.paste()
pyxclip.copy(text.upper())  # or .lower(), .strip(), .replace(...)
print("Text transformed in clipboard")
```

## Type Signatures

```python
def copy(data: None | str | tuple[int, int, bytes] | os.PathLike[str] | os.PathLike[bytes] | list[os.PathLike[str] | str], /) -> None: ...
def paste() -> str | dict[str, object] | list[str]: ...
def clear() -> None: ...

class ClipboardError(RuntimeError): ...
```

## Platform Support

| Platform | Backend | Notes |
|---|---|---|
| Linux | X11 / Wayland | No xclip/xsel. Wayland uses `wayland-data-control`. |
| macOS | NSPasteboard | System framework. |
| Windows | Win32 | System API. |

On headless Linux (no `DISPLAY`, no `WAYLAND_DISPLAY`), all operations raise `ClipboardError`.

## How It Works

A single Rust extension module compiled with PyO3, using [arboard](https://crates.io/crates/arboard) for native clipboard access. No Python dependencies beyond the compiled wheel.

## License

[MIT](LICENSE)
