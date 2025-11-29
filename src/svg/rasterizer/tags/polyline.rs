use std::collections::HashMap;
use crate::svg::parser::tags::Tag;
use crate::svg::rasterizer::canva::Canvas;
use crate::svg::rasterizer::tags::path::draw_path;
use crate::svg::rasterizer::tags::polygon::get_points;
use crate::svg::utils::transform::Transform;

pub fn draw_polyline(
    tag: &mut Tag,
    defs: &HashMap<String, Tag>,
    canvas: &mut Canvas,
    transform: &Transform,
) {
    let points = get_points(tag);

    if points.is_empty() {
        return;
    }

    let mut path_str = String::new();
    
    // Move to the first point
    path_str.push_str(&format!("M {} {} ", points[0].x, points[0].y));

    // Line to subsequent points
    for point in &points[1..] {
        path_str.push_str(&format!("L {} {} ", point.x, point.y));
    }

    let mut temp_tag = tag.clone();
    temp_tag.params.insert("d".to_string(), path_str);

    draw_path(&mut temp_tag, defs, canvas, transform);
}
