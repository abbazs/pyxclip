import pytest
import pyxclip


def test_copy_and_paste_text():
    """Copy text to clipboard and paste it back."""
    original = "Hello, pyxclip!"
    pyxclip.copy_text(original)
    assert pyxclip.paste_text() == original


def test_copy_empty_string():
    """Copying an empty string should work."""
    pyxclip.copy_text("")
    assert pyxclip.paste_text() == ""


def test_unicode_text():
    """Clipboard should handle Unicode/emoji correctly."""
    original = "Hello 🌍 Привет مرحبا"
    pyxclip.copy_text(original)
    assert pyxclip.paste_text() == original


def test_clear_clipboard():
    """Clearing clipboard should make paste fail."""
    pyxclip.copy_text("temporary")
    pyxclip.clear_clipboard()
    with pytest.raises(pyxclip.ClipboardError):
        pyxclip.paste_text()


def test_overwrite_clipboard():
    """Each copy should overwrite the previous content."""
    pyxclip.copy_text("first")
    pyxclip.copy_text("second")
    assert pyxclip.paste_text() == "second"


def test_copy_non_string_raises_type_error():
    """Passing a non-string to copy_text should raise TypeError."""
    with pytest.raises(TypeError):
        pyxclip.copy_text(123)  # type: ignore[arg-type]
