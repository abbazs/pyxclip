# pyxclip

Cross-platform clipboard access with zero external dependencies. No xclip or xsel needed on Linux.

## Installation

```bash
pip install pyxclip
```

## Quick Start

```python
import pyxclip

pyxclip.copy_text("Hello, world!")
text = pyxclip.paste_text()
pyxclip.clear_clipboard()
```

## License

MIT
