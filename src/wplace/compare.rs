use image::{GenericImageView, Pixel};
use pyo3::prelude::*;
use pyo3::types::{PyBytes, PyInt, PyList, PyString, PyTuple};
use std::collections::HashMap;

use crate::utils::spawn_thread_for_async;
use crate::wplace::color_map::find_color_name;
use crate::wplace::utils::load_image;

struct ColorEntry {
    name: &'static str,
    count: usize,
    total: usize,
    pixels: Vec<(usize, usize)>,
}

impl ColorEntry {
    fn new(name: &'static str) -> Self {
        ColorEntry {
            name,
            count: 0,
            total: 0,
            pixels: Vec::new(),
        }
    }

    fn to_py_tuple(&self, py: Python) -> PyResult<Py<PyTuple>> {
        let elements: Vec<Py<PyAny>> = vec![
            PyString::new(py, &self.name).into(),
            PyInt::new(py, self.count).into(),
            PyInt::new(py, self.total).into(),
            PyList::new(py, &self.pixels)?.into(),
        ];
        Ok(PyTuple::new(py, elements)?.into())
    }
}

#[pyfunction]
pub(crate) fn wplace_template_compare(
    template_bytes: &Bound<'_, PyBytes>,
    actual_bytes: &Bound<'_, PyBytes>,
    include_pixels: bool,
    asyncio_loop: &Bound<'_, PyAny>,
) -> PyResult<Py<PyAny>> {
    // 从字节流加载图像
    let template_img = load_image(template_bytes)?;
    let actual_img = load_image(actual_bytes)?;

    spawn_thread_for_async(asyncio_loop, move || {
        // 检查图像尺寸是否匹配
        if template_img.dimensions() != actual_img.dimensions() {
            let msg = "Template and actual images must have the same dimensions.";
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(msg));
        }

        // 转换为 RGBA 格式
        let template_rgba = template_img.to_rgba8();
        let actual_rgba = actual_img.to_rgba8();

        // 获取图像尺寸
        let (width, height) = template_img.dimensions();

        // 创建 diff_pixels 字典
        let mut diff_pixels: HashMap<&'static str, ColorEntry> = HashMap::new();

        // 遍历每个像素
        for y in 0..height {
            for x in 0..width {
                let template_pixel = template_rgba.get_pixel(x, y);

                // 跳过模板中的透明像素
                if template_pixel[3] == 0 {
                    continue;
                }

                // 获取颜色名称
                let color_name = find_color_name(template_pixel);

                // 获取或创建 ColorEntry
                let entry = diff_pixels
                    .entry(color_name)
                    .or_insert_with(|| ColorEntry::new(color_name));

                // 统计模板像素总数
                entry.total += 1;

                // 获取实际像素
                let actual_pixel = actual_rgba.get_pixel(x, y);

                // 如果模板像素颜色与实际像素颜色不同 或 实际像素透明
                if template_pixel.to_rgb() != actual_pixel.to_rgb() || actual_pixel[3] == 0 {
                    entry.count += 1;
                    if include_pixels {
                        entry.pixels.push((x as usize, y as usize));
                    }
                }
            }
        }

        let mut diff_values: Vec<ColorEntry> = diff_pixels.into_values().collect();
        diff_values.sort_by(|a, b| b.total.cmp(&a.total).then_with(|| a.name.cmp(&b.name)));

        Python::attach(|py| -> PyResult<Py<PyList>> {
            let entries = diff_values
                .iter()
                .map(|entry| entry.to_py_tuple(py))
                .collect::<PyResult<Vec<_>>>()?;
            Ok(PyList::new(py, entries)?.into())
        })
    })
}
