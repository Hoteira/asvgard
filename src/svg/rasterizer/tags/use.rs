use crate::utils::compat::{HashMap, ToString, String};
use crate::svg::parser::tags::Tag;
use crate::svg::rasterizer::canva::Canvas;
use crate::svg::utils::transform::Transform;
use crate::svg::utils::coords::parse_length;

pub fn draw_use(
    tag: &mut Tag,
    defs: &HashMap<String, Tag>,
    canvas: &mut Canvas,
    transform: &Transform,
) {
    let href = tag.params.get("href")
        .or_else(|| tag.params.get("xlink:href"));

    if let Some(link) = href {
        let id = link.trim_start_matches('#');
        if let Some(ref_tag) = defs.get(id) {
            let mut cloned_tag = ref_tag.clone();

            let x = tag.params.get("x").map(|s| parse_length(s, 0.0, canvas.width as f32)).unwrap_or(0.0);
            let y = tag.params.get("y").map(|s| parse_length(s, 0.0, canvas.height as f32)).unwrap_or(0.0);
            
            let use_translate = Transform::translate(x, y);
            let mut final_transform = transform.then(&use_translate);

            if let Some(transform_str) = tag.params.get("transform") {
                if let Some(use_transform) = Transform::from_str(transform_str) {
                    final_transform = final_transform.then(&use_transform);
                }
            }
            
            // Basic style inheritance
            if !cloned_tag.params.contains_key("fill") {
                if let Some(fill) = tag.params.get("fill") {
                    cloned_tag.params.insert("fill".to_string(), fill.clone());
                }
            }
            if !cloned_tag.params.contains_key("stroke") {
                if let Some(stroke) = tag.params.get("stroke") {
                    cloned_tag.params.insert("stroke".to_string(), stroke.clone());
                }
            }
             if !cloned_tag.params.contains_key("stroke-width") {
                if let Some(sw) = tag.params.get("stroke-width") {
                    cloned_tag.params.insert("stroke-width".to_string(), sw.clone());
                }
            }

            canvas.draw(&mut cloned_tag, defs, &final_transform);
        }
    }
}
