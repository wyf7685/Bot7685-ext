mod wplace;
use pyo3::prelude::*;

#[pymodule]
#[pyo3(name="_ext")]
fn bot7685_ext(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(wplace::wplace_template_compare, m)?)?;
    Ok(())
}
