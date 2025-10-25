use image::{GenericImageView, Pixel};
use pyo3::{prelude::*, types::{PyBytes, PyInt}};
use std::io::Cursor;

use crate::{utils::spawn_thread_for_async, wplace::utils::load_image};

#[pyfunction]
pub(crate) fn wplace_template_overlay(
    template_bytes: &Bound<'_, PyBytes>,
    actual_bytes: &Bound<'_, PyBytes>,
    overlay_alpha: &Bound<'_, PyInt>,
    asyncio_loop: &Bound<'_, PyAny>,
) -> PyResult<Py<PyAny>> {
    // 从字节流加载图像
    let template_img = load_image(template_bytes)?;
    let actual_img = load_image(actual_bytes)?;
    let overlay_alpha = overlay_alpha.extract::<u8>()?;

    spawn_thread_for_async(asyncio_loop, move || -> PyResult<Py<PyBytes>> {
        // 检查图像尺寸是否匹配
        let (width, height) = template_img.dimensions();
        if (width, height) != actual_img.dimensions() {
            let msg = "Template and actual images must have the same dimensions.";
            return Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(msg));
        }

        let template_rgba = template_img.to_rgba8();
        let mut actual_rgba = actual_img.to_rgba8();

        for y in 0..height {
            for x in 0..width {
                let template_pixel = template_rgba.get_pixel(x, y);
                let actual_pixel = actual_rgba.get_pixel(x, y);
                if template_pixel[3] != 0
                    && (actual_pixel[3] == 0 || template_pixel.to_rgb() != actual_pixel.to_rgb())
                {
                    let new_pixel = image::Rgba([
                        template_pixel[0],
                        template_pixel[1],
                        template_pixel[2],
                        overlay_alpha,
                    ]);
                    actual_rgba.put_pixel(x, y, new_pixel);
                }
            }
        }

        // 将结果图像编码为 PNG 格式
        let mut buffer = Vec::new();
        actual_rgba
            .write_to(&mut Cursor::new(&mut buffer), image::ImageFormat::Png)
            .map_err(|e| {
                let msg = format!("Failed to encode image: {}", e);
                PyErr::new::<pyo3::exceptions::PyValueError, _>(msg)
            })?;

        Python::attach(|py| Ok(PyBytes::new(py, &buffer).into()))
    })
}
