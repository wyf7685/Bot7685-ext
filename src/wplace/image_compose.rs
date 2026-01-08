use image::{Rgba, RgbaImage};
use pyo3::{prelude::*, types::PyBytes};
use std::io::Cursor;

use crate::utils::spawn_thread_for_async;

#[pyfunction]
pub fn wplace_compose_tiles(
    imgs: Vec<((u32, u32), Vec<u8>)>,
    coord1: (u32, u32, u32, u32),
    coord2: (u32, u32, u32, u32),
    background: Option<(u8, u8, u8)>,
    asyncio_loop: &Bound<'_, PyAny>,
) -> PyResult<Py<PyAny>> {
    spawn_thread_for_async(asyncio_loop, move || -> PyResult<Py<PyBytes>> {
        let (tlx1, tly1, pxx1, pxy1) = coord1;
        let (tlx2, tly2, pxx2, pxy2) = coord2;

        // 计算最终图片大小
        let width = (tlx2 - tlx1) * 1000 + pxx2 - pxx1 + 1;
        let height = (tly2 - tly1) * 1000 + pxy2 - pxy1 + 1;

        // 创建背景图片
        let bg_color = match background {
            Some((r, g, b)) => Rgba([r, g, b, 255]),
            None => Rgba([0, 0, 0, 0]),
        };
        let mut result_img = RgbaImage::from_pixel(width, height, bg_color);

        // 遍历所有瓷砖图片
        for ((tx, ty), tile_bytes) in imgs {
            // 解码图片
            let tile_img = image::load_from_memory(&tile_bytes)
                .map_err(|e| {
                    let msg = format!("Failed to load image: {}", e);
                    PyErr::new::<pyo3::exceptions::PyValueError, _>(msg)
                })?
                .to_rgba8();

            // 计算裁剪区域
            let crop_x_start = if tx == tlx1 { pxx1 } else { 0 };
            let crop_y_start = if ty == tly1 { pxy1 } else { 0 };
            let crop_x_end = if tx == tlx2 { pxx2 + 1 } else { 1000 };
            let crop_y_end = if ty == tly2 { pxy2 + 1 } else { 1000 };

            // 计算粘贴位置
            let paste_x = (tx - tlx1) * 1000 - (if tx == tlx1 { 0 } else { pxx1 });
            let paste_y = (ty - tly1) * 1000 - (if ty == tly1 { 0 } else { pxy1 });

            // 裁剪瓷砖图片
            let crop_width = crop_x_end - crop_x_start;
            let crop_height = crop_y_end - crop_y_start;
            (0..crop_height)
                .filter(|cy| paste_y + cy < height)
                .for_each(|cy| {
                    (0..crop_width)
                        .filter(|cx| paste_x + cx < width)
                        .for_each(|cx| {
                            tile_img
                                .get_pixel_checked(crop_x_start + cx, crop_y_start + cy)
                                .filter(|pixel| pixel[3] > 0)
                                .map(|pixel| {
                                    result_img.put_pixel(paste_x + cx, paste_y + cy, *pixel)
                                });
                        });
                });
        }

        // 编码为 PNG
        let mut png_bytes = Vec::new();
        result_img
            .write_to(&mut Cursor::new(&mut png_bytes), image::ImageFormat::Png)
            .map_err(|e| {
                let msg = format!("Failed to encode image: {}", e);
                PyErr::new::<pyo3::exceptions::PyValueError, _>(msg)
            })?;

        Python::attach(|py| Ok(PyBytes::new(py, &png_bytes).into()))
    })
}
