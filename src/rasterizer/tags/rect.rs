use std::collections::HashMap;
use crate::parser::tags::Tag;
use crate::rasterizer::canva::Canvas;
use crate::rasterizer::tags::path::draw_path;
use crate::utils::transform::Transform;

pub fn draw_rect(
    tag: &mut Tag,
    defs: &HashMap<String, Tag>,
    canvas: &mut Canvas,
    transform: &Transform,
) {
    let x = tag.params.get("x").and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
    let y = tag.params.get("y").and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
    let width = tag.params.get("width").and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
    let height = tag.params.get("height").and_then(|s| s.parse::<f32>().ok()).unwrap_or(0.0);
    let mut rx = tag.params.get("rx").and_then(|s| s.parse::<f32>().ok());
    let mut ry = tag.params.get("ry").and_then(|s| s.parse::<f32>().ok());

    if width <= 0.0 || height <= 0.0 {
        return;
    }

    if rx.is_some() && ry.is_none() { ry = rx; }
    if ry.is_some() && rx.is_none() { rx = ry; }

    let rx = rx.unwrap_or(0.0).min(width / 2.0);
    let ry = ry.unwrap_or(0.0).min(height / 2.0);

    let mut path_str = String::new();
    if rx == 0.0 || ry == 0.0 {
        path_str = format!("M {} {} H {} V {} H {} Z",
            x, y,
            x + width,
            y + height,
            x);
    } else {
        path_str.push_str(&format!("M {} {} ", x + rx, y));
        path_str.push_str(&format!("L {} {} ", x + width - rx, y));
        path_str.push_str(&format!("A {} {} 0 0 1 {} {} ", rx, ry, x + width, y + ry));
        path_str.push_str(&format!("L {} {} ", x + width, y + height - ry));
        path_str.push_str(&format!("A {} {} 0 0 1 {} {} ", rx, ry, x + width - rx, y + height));
        path_str.push_str(&format!("L {} {} ", x + rx, y + height));
        path_str.push_str(&format!("A {} {} 0 0 1 {} {} ", rx, ry, x, y + height - ry));
        path_str.push_str(&format!("L {} {} ", x, y + ry));
        path_str.push_str(&format!("A {} {} 0 0 1 {} {} ", rx, ry, x + rx, y));
        path_str.push_str("Z");
    }

    let mut temp_tag = tag.clone();
    temp_tag.params.insert("d".to_string(), path_str);

    draw_path(&mut temp_tag, defs, canvas, transform);
}