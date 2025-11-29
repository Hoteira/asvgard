use minifb::{Window, WindowOptions};
use asvgard::load_image;
use std::time::Instant;

fn main() {
    let data = include_bytes!("../bunny.svg");

    let canvas_width = 600;
    let canvas_height = 600;

    println!("Loading image...");
    let s = Instant::now();
    
    let buffer = match load_image(data, canvas_width, canvas_height) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Failed to load image: {}", e);
            return;
        }
    };
    
    let e = s.elapsed();
    println!("Rendered in: {:?}", e);

    let mut window = Window::new(
        "Asvgard - Image Viewer",
        canvas_width,
        canvas_height,
        WindowOptions::default(),
    ).unwrap_or_else(|e| {
        panic!("{}", e);
    });

    while window.is_open() && !window.is_key_down(minifb::Key::Escape) {
        window.update_with_buffer(&buffer, canvas_width, canvas_height).unwrap();
    }
}
