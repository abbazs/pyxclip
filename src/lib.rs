use std::sync::LazyLock;
use std::sync::Mutex;

use pyo3::create_exception;
use pyo3::exceptions::PyRuntimeError;
use pyo3::prelude::*;

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

#[pyfunction]
fn copy_text(text: &str) -> PyResult<()> {
    with_clipboard(|clipboard| clipboard.set_text(text))
}

#[pyfunction]
fn paste_text() -> PyResult<String> {
    with_clipboard(|clipboard| clipboard.get_text())
}

#[pyfunction]
fn clear_clipboard() -> PyResult<()> {
    with_clipboard(|clipboard| clipboard.clear())
}

#[pymodule]
fn _core(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(copy_text, m)?)?;
    m.add_function(wrap_pyfunction!(paste_text, m)?)?;
    m.add_function(wrap_pyfunction!(clear_clipboard, m)?)?;
    m.add("ClipboardError", m.py().get_type::<ClipboardError>())?;
    m.add("__version__", env!("CARGO_PKG_VERSION"))?;
    Ok(())
}
