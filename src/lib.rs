mod utils;
mod wplace;

use pyo3::prelude::*;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const BUILD_TIME: &str = env!("BUILD_TIME");
pub const GIT_COMMIT_HASH: &str = env!("GIT_COMMIT_HASH");

#[pymodule]
#[pyo3(name = "_ext")]
fn bot7685_ext(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add("__version__", VERSION)?;
    m.add("__build_time__", BUILD_TIME)?;
    m.add("__git_commit_hash__", GIT_COMMIT_HASH)?;
    m.add(
        "WPLACE_COLORS_MAP",
        wplace::COLORS_MAP_VEC.clone().into_pyobject(m.py())?,
    )?;
    m.add_function(wrap_pyfunction!(wplace::wplace_template_compare, m)?)?;
    m.add_function(wrap_pyfunction!(wplace::wplace_template_overlay, m)?)?;
    m.add_function(wrap_pyfunction!(wplace::wplace_group_adjacent, m)?)?;
    m.add_function(wrap_pyfunction!(wplace::wplace_compose_tiles, m)?)?;
    Ok(())
}
