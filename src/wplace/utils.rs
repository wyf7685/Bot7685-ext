use std::io::Cursor;

use pyo3::prelude::*;
use pyo3::types::PyBytes;

pub(crate) use crate::utils::*;

pub(crate) trait LoadableImage {
    fn as_bytes(&self) -> &[u8];
    fn to_image(&self) -> PyResult<image::DynamicImage> {
        image::load_from_memory(self.as_bytes()).map_err(|e| {
            let msg = format!("Failed to load image: {}", e);
            PyErr::new::<pyo3::exceptions::PyValueError, _>(msg)
        })
    }
}

impl LoadableImage for Vec<u8> {
    fn as_bytes(&self) -> &[u8] {
        self
    }
}

impl LoadableImage for &Bound<'_, PyBytes> {
    fn as_bytes(&self) -> &[u8] {
        PyBytesMethods::as_bytes(*self)
    }
}

pub(crate) trait ImageBufferDump {
    fn to_py_png_bytes(&self) -> PyResult<Py<PyBytes>>;
}

impl ImageBufferDump for image::RgbaImage {
    fn to_py_png_bytes(&self) -> PyResult<Py<PyBytes>> {
        let mut buffer = Vec::new();
        self.write_to(&mut Cursor::new(&mut buffer), image::ImageFormat::Png)
            .map_err(|e| {
                let msg = format!("Failed to encode image: {}", e);
                PyErr::new::<pyo3::exceptions::PyValueError, _>(msg)
            })?;
        Python::attach(|py| Ok(PyBytes::new(py, &buffer).into()))
    }
}
