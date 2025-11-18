use pyo3::{
    prelude::*,
    types::{PyFloat, PyInt, PyList},
};
use std::collections::{HashMap, HashSet, VecDeque};

use crate::utils::spawn_thread_for_async;

pub type Point = (i32, i32, i32); // (x, y, color_id)

/// 计算小组的重心坐标
fn calc_group_cxy(group: &[Point]) -> (f64, f64) {
    let len = group.len() as f64;
    let cx = group.iter().map(|(x, _, _)| *x as f64).sum::<f64>() / len;
    let cy = group.iter().map(|(_, y, _)| *y as f64).sum::<f64>() / len;
    (cx, cy)
}

/// 计算两个重心坐标的距离
fn cxy_distance(cxy1: (f64, f64), cxy2: (f64, f64)) -> f64 {
    ((cxy1.0 - cxy2.0).powi(2) + (cxy1.1 - cxy2.1).powi(2)).sqrt()
}

/// BFS 查找连通分组
fn bfs(
    start: (i32, i32),
    point_dict: &HashMap<(i32, i32), i32>,
    visited: &mut HashSet<(i32, i32)>,
) -> Vec<Point> {
    let directions = [
        (-1, -1),
        (-1, 0),
        (-1, 1),
        (0, -1),
        (0, 1),
        (1, -1),
        (1, 0),
        (1, 1),
    ];

    let mut q = VecDeque::new();
    let mut group = Vec::new();

    q.push_back(start);
    visited.insert(start);

    while let Some((x, y)) = q.pop_front() {
        if let Some(&color_id) = point_dict.get(&(x, y)) {
            group.push((x, y, color_id));

            for (dx, dy) in directions.iter() {
                let neighbor = (x + dx, y + dy);
                if point_dict.contains_key(&neighbor) && !visited.contains(&neighbor) {
                    visited.insert(neighbor);
                    q.push_back(neighbor);
                }
            }
        }
    }

    group
}

/// 将相邻点分组，并合并小分组
fn group_adjacent(
    points: Vec<Point>,
    min_group_size: usize,
    merge_distance: f64,
) -> Vec<Vec<Point>> {
    // 构建点字典
    let point_dict: HashMap<(i32, i32), i32> = points
        .iter()
        .map(|(x, y, color_id)| ((*x, *y), *color_id))
        .collect();

    let mut visited = HashSet::new();
    let mut groups = Vec::new();

    // BFS 找出所有连通分组
    for point in &points {
        let (x, y, _) = point;
        if !visited.contains(&(*x, *y)) {
            let group = bfs((*x, *y), &point_dict, &mut visited);
            if !group.is_empty() {
                groups.push(group);
            }
        }
    }

    // 按大小排序（从小到大）
    groups.sort_by_key(|g| g.len());

    // 计算初始重心
    let mut group_cxy = groups
        .iter()
        .map(|g| calc_group_cxy(g))
        .collect::<Vec<(f64, f64)>>();

    // 第一阶段：根据距离合并相邻的小分组
    let mut merged = true;
    while merged {
        merged = false;
        for i in 0..groups.len() {
            if groups[i].len() >= min_group_size {
                continue;
            }
            for j in (i + 1)..groups.len() {
                if cxy_distance(group_cxy[i], group_cxy[j]) <= merge_distance {
                    let group = groups.remove(j);
                    groups[i].extend(group);
                    group_cxy.remove(j);
                    merged = true;
                    break;
                }
            }
            if merged {
                break;
            }
        }
    }

    // 第二阶段：将剩余小分组合并到最近的大分组
    let mut large_group_cxy = Vec::new();
    let mut large_groups = Vec::new();
    let mut small_groups = Vec::new();
    for (idx, group) in groups.into_iter().enumerate() {
        if group.len() >= min_group_size {
            large_group_cxy.push(group_cxy[idx]);
            large_groups.push(group);
        } else {
            small_groups.push(group);
        }
    }
    if large_groups.is_empty() && !small_groups.is_empty() {
        let group = small_groups.remove(0);
        large_group_cxy.push(calc_group_cxy(&group));
        large_groups.push(group);
    }
    for small in small_groups {
        let cxy = calc_group_cxy(&small);
        let closest_large_idx = (0..large_groups.len())
            .min_by_key(|&j| (cxy_distance(cxy, large_group_cxy[j]) * 10000.0) as i64)
            .unwrap();
        large_groups[closest_large_idx].extend(small);
    }

    // 移除空分组并按大小倒序排列
    large_groups.sort_by_cached_key(|g| std::cmp::Reverse(g.len()));
    large_groups
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_calc_group_cxy() {
        let group = vec![(0, 0, 1), (2, 2, 1)];
        let (cx, cy) = calc_group_cxy(&group);
        assert!((cx - 1.0).abs() < 0.001);
        assert!((cy - 1.0).abs() < 0.001);
    }

    #[test]
    fn test_cxy_distance() {
        let dist = cxy_distance((0.0, 0.0), (3.0, 4.0));
        assert!((dist - 5.0).abs() < 0.001);
    }

    #[test]
    fn test_group_adjacent() {
        let points = vec![(0, 0, 1), (1, 1, 1), (2, 2, 1), (10, 10, 2), (11, 11, 2)];
        let groups = group_adjacent(points, 2, 30.0);
        assert_eq!(groups.len(), 2);
    }
}

#[pyfunction]
pub(crate) fn wplace_group_adjacent(
    points: &Bound<'_, PyList>,
    min_group_size: &Bound<'_, PyInt>,
    merge_distance: &Bound<'_, PyFloat>,
    asyncio_loop: &Bound<'_, PyAny>,
) -> PyResult<Py<PyAny>> {
    let points = points
        .iter()
        .map(|item| item.extract::<(i32, i32, i32)>())
        .collect::<PyResult<Vec<Point>>>()?;
    let min_group_size = min_group_size.extract::<usize>()?;
    let merge_distance = merge_distance.extract::<f64>()?;

    spawn_thread_for_async(asyncio_loop, move || -> PyResult<Py<PyList>> {
        let grouped = group_adjacent(points, min_group_size, merge_distance);
        Python::attach(|py| Ok(PyList::new(py, grouped)?.into()))
    })
}
