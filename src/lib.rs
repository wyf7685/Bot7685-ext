mod lru;
mod wplace;

use pyo3::prelude::*;

pub const VERSION: &str = env!("CARGO_PKG_VERSION");
pub const BUILD_TIME: &str = env!("BUILD_TIME");
pub const GIT_COMMIT_HASH: &str = env!("GIT_COMMIT_HASH");

#[pymodule]
pub mod _ext {
    use super::*;

    #[pymodule_export]
    use super::{lru::PyLRU, wplace::py_wplace};

    #[pymodule_init]
    fn module_init(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add("__version__", VERSION)?;
        m.add("__build_time__", BUILD_TIME)?;
        m.add("__git_commit_hash__", GIT_COMMIT_HASH)?;
        Ok(())
    }
}
