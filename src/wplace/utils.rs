use pyo3::prelude::*;
use pyo3::types::PyBytes;

pub(crate) fn load_image(image_bytes: &Bound<'_, PyBytes>) -> PyResult<image::DynamicImage> {
    match image::load_from_memory(image_bytes.as_bytes()) {
        Ok(img) => Ok(img),
        Err(e) => {
            let msg = format!("Failed to load image: {}", e);
            Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(msg))
        }
    }
}
