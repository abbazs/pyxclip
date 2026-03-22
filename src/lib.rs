use std::path::PathBuf;
use std::sync::LazyLock;
use std::sync::Mutex;

use pyo3::create_exception;
use pyo3::exceptions::{PyRuntimeError, PyTypeError, PyValueError};
use pyo3::prelude::*;
use pyo3::types::{PyDict, PyList, PyTuple};
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

fn with_clipboard_file<F>(f: F) -> PyResult<()>
where
    F: FnOnce(&mut arboard::Clipboard) -> Result<(), arboard::Error>,
{
    let mut guard = CLIPBOARD
        .lock()
        .map_err(|e| ClipboardError::new_err(format!("Clipboard lock poisoned: {e}")))?;
    if guard.is_none() {
        *guard = Some(arboard::Clipboard::new().map_err(map_arboard_error)?);
    }
    f(guard.as_mut().unwrap()).map_err(map_file_error)
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

fn map_file_error(err: arboard::Error) -> PyErr {
    match &err {
        arboard::Error::ConversionFailure => ClipboardError::new_err(
            "Failed to copy file paths to clipboard: one or more files may not exist or are inaccessible. \
             All paths must point to existing files or directories.",
        ),
        arboard::Error::ClipboardNotSupported => ClipboardError::new_err(
            "File list clipboard operations are not supported in this environment. \
             This may occur on Wayland without the ext-data-control protocol.",
        ),
        arboard::Error::Unknown { description } => {
            ClipboardError::new_err(format!(
                "Failed to copy file paths to clipboard: {description}"
            ))
        }
        _ => map_arboard_error(err),
    }
}

/// Copy data to the clipboard.
///
/// Accepts any of:
///   - `None` — clears the clipboard
///   - `str` — copies as text
///   - `(width, height, bytes)` — copies as image (RGBA pixels, row-major)
///   - `PathLike` — copies a single file path
///   - `list[PathLike]` — copies multiple file paths
///
/// Note: `str` is always treated as text, never as a file path.
/// Use `pathlib.Path` or `os.fspath()` to copy file paths.
#[pyfunction(signature = (data))]
fn copy(data: &Bound<'_, PyAny>) -> PyResult<()> {
    // None → clear
    if data.is_none() {
        return with_clipboard(|clipboard| clipboard.clear());
    }

    // String → text (str always means text, never a file path)
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

    // Single PathLike → file list with one item
    // (checked after String so str isn't misinterpreted as a path)
    if let Ok(path) = data.extract::<PathBuf>() {
        return with_clipboard_file(|clipboard| clipboard.set().file_list(&[path]));
    }

    // List → file paths
    if let Ok(list) = data.downcast::<PyList>() {
        let mut paths: Vec<PathBuf> = Vec::with_capacity(list.len());
        for item in list.iter() {
            let path: PathBuf = item.extract()?;
            paths.push(path);
        }
        return with_clipboard_file(|clipboard| clipboard.set().file_list(&paths));
    }

    Err(PyTypeError::new_err(
        "copy() expects None, str, (width, height, bytes) tuple, PathLike, or list of paths",
    ))
}

/// Paste content from the clipboard.
///
/// Returns:
///   - `str` if the clipboard contains text
///   - `dict` with keys "width", "height", "bytes" if the clipboard contains an image
///   - `list[str]` if the clipboard contains file paths
///
/// Raises `ClipboardError` if the clipboard is empty or inaccessible.
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
    if let Ok(text) = clipboard.get_text() {
        let py_str = text.into_pyobject(py)?;
        return Ok(py_str.into_any().unbind());
    }

    // Try image
    if let Ok(img) = clipboard.get_image() {
        let dict = PyDict::new(py);
        dict.set_item("width", img.width)?;
        dict.set_item("height", img.height)?;
        dict.set_item("bytes", img.into_owned_bytes().into_owned())?;
        return Ok(dict.into_any().unbind());
    }

    // Try file list
    if let Ok(paths) = clipboard.get().file_list() {
        let py_list = PyList::empty(py);
        for path in paths {
            let path_str = path.to_string_lossy().to_string();
            py_list.append(path_str)?;
        }
        return Ok(py_list.into_any().unbind());
    }

    Err(ClipboardError::new_err(
        "Clipboard is empty or contains incompatible data",
    ))
}

/// Clear the clipboard.
#[pyfunction]
fn clear() -> PyResult<()> {
    with_clipboard(|clipboard| clipboard.clear())
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(copy, m)?)?;
    m.add_function(wrap_pyfunction!(paste, m)?)?;
    m.add_function(wrap_pyfunction!(clear, m)?)?;
    m.add("ClipboardError", m.py().get_type::<ClipboardError>())?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
