mod parser;
mod rasterizer;
mod utils;

use std::collections::HashMap;
use std::time::Instant;
use minifb;
use minifb::{Window, WindowOptions};
use crate::parser::tags::Tag;
use crate::utils::get_id;
use crate::utils::transform::Transform;

fn main() {
    let svg_data = include_bytes!("../bunny.svg");

    let canvas_width = 600;
    let canvas_height = 500;

    let mut window = Window::new(
        "Asvgard - SVG Rasterizer",
        canvas_width,
        canvas_height,
        WindowOptions::default(),
    ).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    let mut svg_tags = parser::parse::load_xml(svg_data);
    let mut canva = rasterizer::canva::Canvas::new(canvas_width, canvas_height);

    let mut defs_map: HashMap<String, Tag> = HashMap::new();
    traverse_recursive(&mut defs_map, &svg_tags[0]);

    let transform = get_svg_transform(&svg_tags[0], canvas_width, canvas_height);

    let s = Instant::now();
    for tag in &mut svg_tags {
        canva.draw(tag, &defs_map, &transform);
    }
    let e = s.elapsed();
    println!("Time elapsed: {:?}", e);

    println!("Rendered!");

    loop {
        window.update_with_buffer(&canva.data, canvas_width, canvas_height).unwrap();
    }
}

pub fn traverse_recursive(defs: &mut HashMap<String, Tag>, start: &Tag) {
    if get_id(start).is_some() {
        defs.insert(get_id(start).unwrap().clone(), start.clone());
    }

    for child in &start.children {
        traverse_recursive(defs, child);
    }
}

pub fn get_svg_transform(svg_tag: &Tag, canvas_width: usize, canvas_height: usize) -> Transform {
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