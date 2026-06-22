mod color_map;
mod compare;
mod group;
mod image_compose;
mod overlay;
mod utils;

use pyo3::prelude::*;

pub(crate) use color_map::COLORS_MAP_VEC;

#[pymodule(name = "wplace")]
pub mod py_wplace {
    use super::*;

    #[pymodule_export]
    use super::{
        compare::wplace_template_compare, group::wplace_group_adjacent,
        image_compose::wplace_compose_tiles, overlay::wplace_template_overlay,
    };

    #[pymodule_init]
    fn module_init(m: &Bound<'_, PyModule>) -> PyResult<()> {
        m.add("COLORS_MAP", COLORS_MAP_VEC.clone().into_pyobject(m.py())?)?;
        Ok(())
    }
}
