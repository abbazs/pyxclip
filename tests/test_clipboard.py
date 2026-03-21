import os

import pytest
import pyxclip

skip_no_display = pytest.mark.skipif(
    not os.environ.get("DISPLAY") and not os.environ.get("WAYLAND_DISPLAY"),
    reason="No display server available (headless CI)",
)


@skip_no_display
def test_copy_and_paste_text():
    original = "Hello, pyxclip!"
    pyxclip.copy(original)
    assert pyxclip.paste() == original


@skip_no_display
def test_copy_empty_string():
    pyxclip.copy("")
    assert pyxclip.paste() == ""


@skip_no_display
def test_unicode_text():
    original = "Hello 🌍 Привет مرحبا"
    pyxclip.copy(original)
    assert pyxclip.paste() == original


@skip_no_display
def test_clear_clipboard():
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
    pyxclip.copy_image(width, height, rgba)


@skip_no_display
def test_copy_image_via_tuple():
    width, height = 2, 2
    rgba = bytes([255, 0, 0, 255, 0, 255, 0, 255, 0, 0, 255, 255, 255, 255, 255, 255])
    pyxclip.copy((width, height, rgba))


@skip_no_display
def test_copy_files():
    pyxclip.copy_files([__file__])


def test_old_api_removed():
    assert not hasattr(pyxclip, "copy_text")
    assert not hasattr(pyxclip, "paste_text")
    assert not hasattr(pyxclip, "clear_clipboard")
