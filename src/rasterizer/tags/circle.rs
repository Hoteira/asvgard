use std::collections::HashMap;
use crate::parser::tags::Tag;
use crate::rasterizer::canva::Canvas;
use crate::rasterizer::tags::ellipse::draw_ellipse;
use crate::utils::transform::Transform;

pub fn draw_circle(
    tag: &mut Tag,
    defs: &HashMap<String, Tag>,
    canvas: &mut Canvas,
    transform: &Transform,
) {
    if let Some(r_str) = tag.params.get("r") {
        let mut temp_tag = tag.clone();
        temp_tag.params.insert("rx".to_string(), r_str.clone());
        temp_tag.params.insert("ry".to_string(), r_str.clone());
        draw_ellipse(&mut temp_tag, defs, canvas, transform);
    }
}
