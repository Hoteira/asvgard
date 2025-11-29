use crate::utils::compat::{HashMap, ToString, format, String};
use crate::svg::parser::tags::Tag;
use crate::svg::rasterizer::canva::Canvas;
use crate::svg::rasterizer::tags::path::draw_path;
use crate::svg::utils::transform::Transform;

pub fn draw_line(
    tag: &mut Tag,
    defs: &HashMap<String, Tag>,
    canvas: &mut Canvas,
    transform: &Transform,
) {
    let x1 = tag.params.get("x1").and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
    let y1 = tag.params.get("y1").and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
    let x2 = tag.params.get("x2").and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
    let y2 = tag.params.get("y2").and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);

    let path_str = format!("M {} {} L {} {}", x1, y1, x2, y2);

    let mut temp_tag = tag.clone();
    temp_tag.params.insert("d".to_string(), path_str);

    draw_path(&mut temp_tag, defs, canvas, transform);
}
