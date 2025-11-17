
mod parser;
mod rasterizer;
mod utils;

use minifb;
use minifb::{Window, WindowOptions};

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

    for tag in &svg_tags {
        println!("Tag: {}, children: {}", tag.name, tag.children.len());
        for child in &tag.children {
            println!("  Child: {}", child.name);
        }
    }

    for tag in &mut svg_tags {
        canva.draw(tag);
    }


    println!("Hello, world!");

    loop {
        window.update_with_buffer(&canva.data, 1024, 1024).unwrap();
    }
}
