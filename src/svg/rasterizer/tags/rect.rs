use crate::utils::compat::{HashMap, String, ToString, format};
use crate::utils::compat::FloatExt;
use crate::svg::parser::tags::Tag;
use crate::svg::rasterizer::canva::Canvas;
use crate::svg::rasterizer::tags::path::draw_path;
use crate::svg::utils::coords::parse_length;
use crate::svg::utils::transform::Transform;

pub fn draw_rect(
    tag: &mut Tag,
    defs: &HashMap<String, Tag>,
    canvas: &mut Canvas,
    transform: &Transform,
) {
    let canvas_w = canvas.width as f32;
    let canvas_h = canvas.height as f32;

    let x = tag.params.get("x").map(|s| parse_length(s, 0.0, canvas_w)).unwrap_or(0.0);
    let y = tag.params.get("y").map(|s| parse_length(s, 0.0, canvas_h)).unwrap_or(0.0);
    let width = tag.params.get("width").map(|s| parse_length(s, 0.0, canvas_w)).unwrap_or(0.0);
    let height = tag.params.get("height").map(|s| parse_length(s, 0.0, canvas_h)).unwrap_or(0.0);
    
    let mut rx = tag.params.get("rx").map(|s| parse_length(s, 0.0, canvas_w));
    let mut ry = tag.params.get("ry").map(|s| parse_length(s, 0.0, canvas_h));

    if width <= 0.0 || height <= 0.0 {
        return;
    }

    // If one is set but not the other, use the set one for both
    // Note: logic differs slightly from original but conforms to SVG: if rx present and ry missing, ry=rx.
    if tag.params.contains_key("rx") && !tag.params.contains_key("ry") {
        ry = rx;
    } else if !tag.params.contains_key("rx") && tag.params.contains_key("ry") {
        rx = ry;
    }
    
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