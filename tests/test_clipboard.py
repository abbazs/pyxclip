import os
from pathlib import Path

import pytest
import pyxclip

skip_no_display = pytest.mark.skipif(
    not os.environ.get("DISPLAY") and not os.environ.get("WAYLAND_DISPLAY"),
    reason="No display server available (headless CI)",
)


@skip_no_display
def test_copy_and_paste_text():
    pyxclip.copy("Hello, pyxclip!")
    assert pyxclip.paste() == "Hello, pyxclip!"


@skip_no_display
def test_copy_empty_string():
    pyxclip.copy("")
    assert pyxclip.paste() == ""


@skip_no_display
def test_unicode_text():
    pyxclip.copy("Hello 🌍 Привет مرحبا")
    assert pyxclip.paste() == "Hello 🌍 Привет مرحبا"


@skip_no_display
def test_clear_via_none():
    pyxclip.copy("temporary")
    pyxclip.copy(None)
    with pytest.raises(pyxclip.ClipboardError):
        pyxclip.paste()


@skip_no_display
def test_clear():
    pyxclip.copy("temporary")
    pyxclip.clear()
    with pytest.raises(pyxclip.ClipboardError):
        pyxclip.paste()


@skip_no_display
def test_overwrite_clipboard():
    pyxclip.copy("first")
    pyxclip.copy("second")
    assert pyxclip.paste() == "second"


def test_copy_non_string_raises_type_error():
    with pytest.raises(TypeError):
        pyxclip.copy(123)  # type: ignore[arg-type]


@skip_no_display
def test_copy_image():
    width, height = 2, 2
    rgba = bytes([255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255])
    pyxclip.copy((width, height, rgba))


@skip_no_display
def test_copy_files():
    pyxclip.copy([__file__])


@skip_no_display
def test_copy_single_path():
    pyxclip.copy(Path(__file__))


@skip_no_display
def test_copy_single_path_str():
    pyxclip.copy(os.fspath(__file__))


@skip_no_display
def test_copy_files_with_pathlib_list():
    pyxclip.copy([Path(__file__), Path(__file__)])


@skip_no_display
def test_copy_nonexistent_file_gives_clear_error():
    with pytest.raises(pyxclip.ClipboardError, match="Cannot resolve path"):
        pyxclip.copy(Path("/nonexistent/path/that/does/not/exist.txt"))


@skip_no_display
def test_copy_nonexistent_file_list_gives_clear_error():
    with pytest.raises(pyxclip.ClipboardError, match="Cannot resolve path"):
        pyxclip.copy([Path("/nonexistent/path/that/does/not/exist.txt")])


@skip_no_display
def test_str_is_never_treated_as_path():
    pyxclip.copy("/some/random/path.txt")
    assert pyxclip.paste() == "/some/random/path.txt"


@skip_no_display
def test_copy_files_roundtrip():
    pyxclip.copy([__file__])
    result = pyxclip.paste()
    assert isinstance(result, list), f"Expected list, got {type(result).__name__}"
    assert len(result) == 1
    assert result[0] == Path(__file__).resolve().as_posix()


@skip_no_display
def test_copy_single_path_roundtrip():
    pyxclip.copy(Path(__file__))
    result = pyxclip.paste()
    assert isinstance(result, list), f"Expected list, got {type(result).__name__}"
    assert len(result) == 1
    assert result[0] == Path(__file__).resolve().as_posix()


@skip_no_display
def test_copy_relative_path_becomes_absolute():
    import tempfile

    with tempfile.NamedTemporaryFile(suffix=".txt", delete=False) as f:
        f.write(b"test")
        tmp_path = f.name
    try:
        pyxclip.copy(Path(tmp_path))
        result = pyxclip.paste()
        assert isinstance(result, list)
        assert result[0] == Path(tmp_path).resolve().as_posix()
    finally:
        os.unlink(tmp_path)


def test_old_api_removed():
    assert not hasattr(pyxclip, "copy_text")
    assert not hasattr(pyxclip, "paste_text")
    assert not hasattr(pyxclip, "clear_clipboard")
    assert not hasattr(pyxclip, "copy_image")
    assert not hasattr(pyxclip, "copy_files")
