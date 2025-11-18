use std::collections::HashMap;
use crate::parser::tags::Tag;
use crate::rasterizer::tags::path::draw_path;
use crate::rasterizer::tags::rect::draw_rect;

pub struct Canvas {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) data: Vec<u32>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![0; width * height],
        }
    }
}

impl Canvas {
    pub fn draw(&mut self, tag: &mut Tag, defs: &HashMap<String, Tag>, scale: f32, offset_x: f32, offset_y: f32) {
        match &*tag.name {
            //"rect" => { draw_rect(tag, defs, self, scale_x, scale_y, offset_x, offset_y) }
            "path" => { draw_path(tag, defs, self, scale, offset_x, offset_y); }
            _ => {}
        }

        let children = tag.children.len();

        for i in 0..children {
            self.draw(&mut tag.children[i], defs, scale, offset_x, offset_y);
        }
    }

    pub fn draw_buffer(
        &mut self,
        bitmap: &[u32],
        x: i32,
        y: i32,
        bitmap_width: usize,
        bitmap_height: usize
    ) {
        for i in 0..bitmap_height {
            for j in 0..bitmap_width {
                let screen_x = x + j as i32;
                let screen_y = y + i as i32;

                if screen_x < 0 || screen_x >= self.width as i32 ||
                    screen_y < 0 || screen_y >= self.height as i32 {
                    continue;
                }

                let coverage = ((bitmap[i * bitmap_width + j] >> 24) & 0xFF) as u8;
                let color = bitmap[i * bitmap_width + j] & 0xFFFFFF;
                if coverage == 0 { continue; }

                let screen_idx = (screen_y as usize) * self.width + (screen_x as usize);

                if coverage == 255 {
                    self.data[screen_idx] = bitmap[i * bitmap_width + j];
                } else {
                    self.data[screen_idx] = blend_coverage(
                        self.data[screen_idx],
                        color,
                        coverage
                    );
                }
            }
        }
    }
}

fn blend_coverage(background: u32, foreground: u32, coverage: u8) -> u32 {
    let alpha = coverage as f32 / 255.0;

    let bg_r = ((background >> 16) & 0xFF) as f32;
    let bg_g = ((background >> 8) & 0xFF) as f32;
    let bg_b = (background & 0xFF) as f32;

    let fg_r = ((foreground >> 16) & 0xFF) as f32;
    let fg_g = ((foreground >> 8) & 0xFF) as f32;
    let fg_b = (foreground & 0xFF) as f32;

    let out_r = (fg_r * alpha + bg_r * (1.0 - alpha)) as u32;
    let out_g = (fg_g * alpha + bg_g * (1.0 - alpha)) as u32;
    let out_b = (fg_b * alpha + bg_b * (1.0 - alpha)) as u32;

    (out_r << 16) | (out_g << 8) | out_b
}


