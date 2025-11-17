
mod parser;
mod rasterizer;
mod utils;

use std::collections::HashMap;
use minifb;
use minifb::{Window, WindowOptions};
use crate::parser::tags::Tag;
use crate::utils::get_id;

fn main() {

    let svg_data = include_bytes!("../icon.svg");

    let mut window = Window::new(
        "Asvgard - SVG Rasterizer",
        1024,
        1024,
        WindowOptions::default(),
    )
        .unwrap_or_else(|e| {
            panic!("{}", e);
        });

    let mut svg_tags = parser::parse::load_xml(svg_data);
    let mut canva = rasterizer::canva::Canvas::new(1024, 1024);

    let mut defs_map: HashMap<String, Tag> = HashMap::new();

    traverse_recursive(&mut defs_map, &svg_tags[0]);

    println!("{:?}", defs_map);

    for tag in &mut svg_tags {
        canva.draw(tag, &defs_map);
    }


    println!("Hello, world!");

    loop {
        window.update_with_buffer(&canva.data, 1024, 1024).unwrap();
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
