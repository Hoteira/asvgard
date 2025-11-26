use std::collections::HashMap;
use crate::parser::tags::Tag;
use crate::rasterizer::canva::Canvas;
use crate::rasterizer::tags::path::draw_path;
use crate::utils::transform::Transform;

pub fn draw_ellipse(
    tag: &mut Tag,
    defs: &HashMap<String, Tag>,
    canvas: &mut Canvas,
    transform: &Transform,
) {
    let cx = tag.params.get("cx").and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
    let cy = tag.params.get("cy").and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
    let rx = tag.params.get("rx").and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
    let ry = tag.params.get("ry").and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);

    if rx <= 0.0 || ry <= 0.0 {
        return;
    }

    let path_str = format!("M {} {} a {} {} 0 1 0 {} 0 a {} {} 0 1 0 {} 0",
        cx - rx, cy,
        rx, ry,
        rx * 2.0,
        rx, ry,
        -rx * 2.0);

    let mut temp_tag = tag.clone();
    temp_tag.params.insert("d".to_string(), path_str);

    draw_path(&mut temp_tag, defs, canvas, transform);
}
