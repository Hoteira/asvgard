use crate::utils::compat::{HashMap, Vec, String, ToString, format};
use crate::svg::parser::tags::Tag;
use crate::svg::rasterizer::canva::Canvas;
use crate::svg::rasterizer::tags::path::draw_path;
use crate::svg::utils::transform::Transform;

pub fn draw_polygon(tag: &mut Tag, defs: &HashMap<String, Tag>, canvas: &mut Canvas, transform: &Transform) {
    let points = get_points(tag);

    if points.is_empty() {
        return;
    }

    let mut path_str = String::new();
    
    path_str.push_str(&format!("M {} {} ", points[0].x, points[0].y));
    for point in &points[1..] {
        path_str.push_str(&format!("L {} {} ", point.x, point.y));
    }
    path_str.push_str("Z");

    let mut temp_tag = tag.clone();
    temp_tag.params.insert("d".to_string(), path_str);

    draw_path(&mut temp_tag, defs, canvas, transform);
}

#[inline]
pub fn get_points(tag: &Tag) -> Vec<crate::svg::rasterizer::raster::Point> {
    let Some(points_str) = tag.params.get("points") else {
        return Vec::new();
    };

    let nums: Vec<f32> = points_str
        .split(|c: char| c == ',' || c.is_whitespace())
        .filter(|s| !s.is_empty())
        .filter_map(|s| s.parse::<f32>().ok())
        .collect();

    nums.chunks_exact(2)
        .map(|chunk| crate::svg::rasterizer::raster::Point {
            x: chunk[0],
            y: chunk[1]
        })
        .collect()
}