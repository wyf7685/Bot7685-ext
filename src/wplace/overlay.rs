use image::{GenericImageView, Pixel};
use pyo3::prelude::*;

use crate::wplace::utils::*;

#[pyfunction]
pub(crate) fn wplace_template_overlay(
    template_bytes: Vec<u8>,
    actual_bytes: Vec<u8>,
    overlay_alpha: u8,
    asyncio_loop: &Bound<'_, PyAny>,
) -> PyResult<Py<PyAny>> {
    spawn_thread_for_async(asyncio_loop, move || -> PyResult<_> {
        // 从字节流加载图像
        let template_img = template_bytes.to_image()?;
        let actual_img = actual_bytes.to_image()?;

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
        actual_rgba.to_py_png_bytes()
    })
}
