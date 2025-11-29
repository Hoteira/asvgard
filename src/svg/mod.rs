//! SVG parsing and rasterization.
//! 
//! Handles parsing of SVG XML data into a tree of tags, and then rasterizing 
//! those tags (paths, shapes) onto a pixel buffer.

pub mod parser;
pub mod rasterizer;
pub mod utils;

use std::collections::HashMap;
use crate::svg::parser::tags::Tag;
use crate::svg::utils::transform::Transform;
use crate::svg::utils::get_id;
use crate::svg::rasterizer::canva::Canvas;

/// Renders an SVG byte stream into a pixel buffer.
///
/// # Arguments
///
/// * `data` - Raw SVG file bytes (XML text).
/// * `width` - Target width.
/// * `height` - Target height.
pub fn render(data: &[u8], width: usize, height: usize) -> Result<Vec<u32>, String> {
    let mut svg_tags = parser::parse::load_xml(data);
    if svg_tags.is_empty() {
        return Err("No SVG tags found or invalid XML".to_string());
    }
    
    // Render at requested resolution directly
    let mut canvas = Canvas::new(width, height);
    let mut defs_map: HashMap<String, Tag> = HashMap::new();
    
    traverse_recursive(&mut defs_map, &svg_tags[0]);
    
    let transform = get_svg_transform(&svg_tags[0], width, height);
    
    for tag in &mut svg_tags {
        canvas.draw(tag, &defs_map, &transform);
    }
    
    Ok(canvas.data)
}

fn traverse_recursive(defs: &mut HashMap<String, Tag>, start: &Tag) {
    if let Some(id) = get_id(start) {
        defs.insert(id.clone(), start.clone());
    }

    for child in &start.children {
        traverse_recursive(defs, child);
    }
}

fn get_svg_transform(svg_tag: &Tag, canvas_width: usize, canvas_height: usize) -> Transform {
    let viewbox = svg_tag.params.get("viewBox");

    if let Some(vb) = viewbox {
        let parts: Vec<f32> = vb.split_whitespace()
            .filter_map(|s| s.parse().ok())
            .collect();

        if parts.len() == 4 {
            let vb_x = parts[0];
            let vb_y = parts[1];
            let vb_width = parts[2];
            let vb_height = parts[3];

            let scale_x = canvas_width as f32 / vb_width;
            let scale_y = canvas_height as f32 / vb_height;
            let scale = scale_x.min(scale_y);

            return Transform::scale(scale, scale)
                .then(&Transform::translate(-vb_x * scale, -vb_y * scale));
        }
    }

    let svg_width = svg_tag.params.get("width")
        .and_then(|w| w.parse::<f32>().ok())
        .unwrap_or(canvas_width as f32);
    let svg_height = svg_tag.params.get("height")
        .and_then(|h| h.parse::<f32>().ok())
        .unwrap_or(canvas_height as f32);

    let scale_x = canvas_width as f32 / svg_width;
    let scale_y = canvas_height as f32 / svg_height;

    Transform::scale(scale_x, scale_y)
}
