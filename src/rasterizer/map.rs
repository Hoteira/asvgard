use std::collections::HashMap;
use crate::parser::tags::Tag;
use crate::rasterizer::tags::path::draw_path;
use crate::rasterizer::tags::polygon::draw_polygon;
use crate::rasterizer::tags::rect::draw_rect;
use crate::utils::transform::{parse_transform, Transform};

pub struct Map {
    width: usize,
    height: usize,
    pub pixels: Vec<Pixel>,
}

#[derive(Clone, Copy)]
pub struct Pixel {
    r: f32,
    g: f32,
    b: f32,
    a: f32,
}

impl Pixel {
    #[inline]
    pub fn to_color(&self) -> u32 {
        let a = (self.a.clamp(0.0, 1.0) * 255.0) as u32;

        // unpremultiply RGB
        let r = if self.a > 0.0 { (self.r / self.a).clamp(0.0, 255.0) as u32 } else { 0 };
        let g = if self.a > 0.0 { (self.g / self.a).clamp(0.0, 255.0) as u32 } else { 0 };
        let b = if self.a > 0.0 { (self.b / self.a).clamp(0.0, 255.0) as u32 } else { 0 };

        (a << 24) | (r << 16) | (g << 8) | b
    }
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        Self { width, height, pixels: vec![Pixel { r: 0.0, g: 0.0, b: 0.0, a: 1.0 }; width * height] }
    }

    #[inline]
    pub fn contribute(&mut self, idx: usize, color: u32) {
        let a = ((color >> 24) & 0xFF) as f32 / 255.0;
        let r = ((color >> 16) & 0xFF) as f32 * a;
        let g = ((color >> 8) & 0xFF) as f32 * a;
        let b = (color & 0xFF) as f32 * a;

        let pixel = &mut self.pixels[idx];

        if a >= 0.9 {
            pixel.r = r;
            pixel.g = g;
            pixel.b = b;
            pixel.a = a;
        } else {
            pixel.r += r;
            pixel.g += g;
            pixel.b += b;
            pixel.a += a;
        }
    }


    pub fn draw(&mut self, tag: &mut Tag, defs: &HashMap<String, Tag>, transform: &Transform) {

        match &*tag.name {
            "clipPath" | "defs" | "linearGradient" | "radialGradient" |
            "pattern" | "mask" | "marker" | "filter" => return,

            _ => {}
        }

        let local_transform = parse_transform(tag);
        let combined = transform.then(&local_transform);

        match tag.name.as_str() {
            "path" => { draw_path(tag, defs, self, &combined); }
            "rect" => { draw_rect(tag, defs, self, &combined); }
            "polygon" => { draw_polygon(tag, defs, self, &combined); }
            _ => {}
        }

        for child in &mut tag.children {
            self.draw(child, defs, &combined);
        }
    }

    pub fn add_buffer(
        &mut self,
        color_map: &[u32],
        x: usize,
        y: usize,
        colormap_width: usize,
        colormap_height: usize
    ) {

        for i in 0..colormap_height {
            for j in 0..colormap_width {
                let screen_x = x + j;
                let screen_y = y + i;

                if screen_x >= self.width || screen_y >= self.height {
                    continue;
                }

                let screen_idx = (screen_y as usize) * self.width + (screen_x as usize);
                self.contribute(screen_idx, color_map[i * colormap_width + j]);
            }
        }
    }
}