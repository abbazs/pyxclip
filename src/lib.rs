use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::Mutex;

use pyo3::create_exception;
use pyo3::exceptions::{PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyTuple};
use pyo3::IntoPyObject;

create_exception!(pyxclip, ClipboardError, PyRuntimeError);

static CLIPBOARD: LazyLock<Mutex<Option<arboard::Clipboard>>> = LazyLock::new(|| Mutex::new(None));

fn with_clipboard<F, R>(f: F) -> PyResult<R>
where
    F: FnOnce(&mut arboard::Clipboard) -> Result<R, arboard::Error>,
{
    let mut guard = CLIPBOARD
        .lock()
        .map_err(|e| ClipboardError::new_err(format!("Clipboard lock poisoned: {e}")))?;
    if guard.is_none() {
        *guard = Some(arboard::Clipboard::new().map_err(map_arboard_error)?);
    }
    f(guard.as_mut().unwrap()).map_err(map_arboard_error)
}

fn map_arboard_error(err: arboard::Error) -> PyErr {
    let description = match &err {
        arboard::Error::ContentNotAvailable => {
            "Clipboard is empty or contains incompatible data".to_string()
        }
        arboard::Error::ClipboardNotSupported => {
            "Clipboard is not supported in this environment".to_string()
        }
        arboard::Error::ClipboardOccupied => {
            "Clipboard is held by another process; retry later".to_string()
        }
        arboard::Error::ConversionFailure => {
            "Failed to convert clipboard data to the requested format".to_string()
        }
        arboard::Error::Unknown { description } => {
            if description.contains("unreachable") || description.contains("not found") {
                format!("No display server available (headless?): {description}")
            } else {
                format!("Clipboard error: {description}")
            }
        }
        _ => err.to_string(),
    };
    ClipboardError::new_err(description)
}

/// Copy data to the clipboard.
///
/// Accepts:
///   - str: copies as text
///   - bytes: copies as image (must be RGBA pixels with explicit width/height via `copy_image`)
///   - tuple (width, height, bytes): copies as image
///   - str or PathLike: copies as file path (not supported on all platforms)
#[pyfunction]
fn copy(data: &Bound<'_, PyAny>) -> PyResult<()> {
    // String → text
    if let Ok(text) = data.extract::<String>() {
        return with_clipboard(|clipboard| clipboard.set_text(text));
    }

    // Tuple (width, height, bytes) → image
    if let Ok(tuple) = data.downcast::<PyTuple>() {
        if tuple.len() == 3 {
            let width: usize = tuple.get_item(0)?.extract()?;
            let height: usize = tuple.get_item(1)?.extract()?;
            let py_bytes: &Bound<'_, PyAny> = &tuple.get_item(2)?;
            let py_bytes: &[u8] = py_bytes.extract()?;
            let image = arboard::ImageData {
                width,
                height,
                bytes: std::borrow::Cow::Borrowed(py_bytes),
            };
            return with_clipboard(|clipboard| clipboard.set_image(image));
        }
        return Err(PyValueError::new_err(
            "Image tuple must be (width: int, height: int, pixels: bytes)",
        ));
    }

    Err(PyTypeError::new_err(
        "copy() expects str, bytes (via copy_image), or (width, height, bytes) tuple",
    ))
}

/// Copy an image to the clipboard.
///
/// Args:
///   width: Image width in pixels
///   height: Image height in pixels
///   rgba_bytes: Raw RGBA pixel data as bytes (4 bytes per pixel, row-major)
#[pyfunction]
fn copy_image(width: usize, height: usize, rgba_bytes: &[u8]) -> PyResult<()> {
    let image = arboard::ImageData {
        width,
        height,
        bytes: std::borrow::Cow::Borrowed(rgba_bytes),
    };
    with_clipboard(|clipboard| clipboard.set_image(image))
}

/// Copy a list of file paths to the clipboard.
#[pyfunction]
fn copy_files(_py: Python, paths: Vec<PathBuf>) -> PyResult<()> {
    with_clipboard(|clipboard| {
        clipboard
            .set()
            .file_list(&paths)
            .map_err(|_| arboard::Error::ClipboardNotSupported)
    })
}

/// Paste content from the clipboard.
///
/// Returns the clipboard content as text. If the clipboard contains an image,
/// returns a dict with keys "width", "height", "bytes".
#[pyfunction]
fn paste(py: Python) -> PyResult<PyObject> {
    let mut clipboard = CLIPBOARD
        .lock()
        .map_err(|e| ClipboardError::new_err(format!("Clipboard lock poisoned: {e}")))?;
    if clipboard.is_none() {
        *clipboard = Some(arboard::Clipboard::new().map_err(map_arboard_error)?);
    }
    let clipboard = clipboard.as_mut().unwrap();

    // Try text first (most common case)
    match clipboard.get_text() {
        Ok(text) => {
            let py_str = text.into_pyobject(py)?;
            Ok(py_str.into_any().unbind())
        }
        Err(arboard::Error::ContentNotAvailable) => match clipboard.get_image() {
            Ok(img) => {
                let dict = PyDict::new(py);
                dict.set_item("width", img.width)?;
                dict.set_item("height", img.height)?;
                dict.set_item("bytes", img.into_owned_bytes().into_owned())?;
                let obj = dict.into_any();
                Ok(obj.unbind())
            }
            Err(_) => Err(arboard::Error::ContentNotAvailable),
        },
        Err(e) => Err(e),
    }
    .map_err(map_arboard_error)
}

/// Clear the clipboard.
#[pyfunction]
fn clear() -> PyResult<()> {
    with_clipboard(|clipboard| clipboard.clear())
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(copy, m)?)?;
    m.add_function(wrap_pyfunction!(copy_image, m)?)?;
    m.add_function(wrap_pyfunction!(copy_files, m)?)?;
    m.add_function(wrap_pyfunction!(paste, m)?)?;
    m.add_function(wrap_pyfunction!(clear, m)?)?;
    m.add("ClipboardError", m.py().get_type::<ClipboardError>())?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
