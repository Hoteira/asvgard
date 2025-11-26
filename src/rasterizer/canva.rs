use std::collections::HashMap;
use crate::parser::tags::Tag;
use crate::rasterizer::tags::path::draw_path;
use crate::rasterizer::tags::polygon::draw_polygon;
use crate::rasterizer::tags::rect::draw_rect;
use crate::rasterizer::tags::circle::draw_circle;
use crate::rasterizer::tags::ellipse::draw_ellipse;
use crate::utils::transform::{parse_transform, Transform};

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
            data: vec![0xFFFF_FFFF; width * height],
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
            "circle" => { draw_circle(tag, defs, self, &combined); }
            "ellipse" => { draw_ellipse(tag, defs, self, &combined); }
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
                self.data[screen_idx] = self.contribute(screen_idx, color_map[i * colormap_width + j]);
            }
        }
    }

    #[inline]
    pub fn contribute(&mut self, idx: usize, color: u32) -> u32 {
        let src_a = ((color >> 24) & 0xFF) as f32 / 255.0;
        let src_r = ((color >> 16) & 0xFF) as f32;
        let src_g = ((color >> 8) & 0xFF) as f32;
        let src_b = (color & 0xFF) as f32;

        let dst_color = self.data[idx];
        let dst_a = ((dst_color >> 24) & 0xFF) as f32 / 255.0;
        let dst_r = ((dst_color >> 16) & 0xFF) as f32;
        let dst_g = ((dst_color >> 8) & 0xFF) as f32;
        let dst_b = (dst_color & 0xFF) as f32;

        let out_a = src_a + dst_a * (1.0 - src_a);
        let out_r = (src_r * src_a + dst_r * dst_a * (1.0 - src_a)) / out_a.max(0.001);
        let out_g = (src_g * src_a + dst_g * dst_a * (1.0 - src_a)) / out_a.max(0.001);
        let out_b = (src_b * src_a + dst_b * dst_a * (1.0 - src_a)) / out_a.max(0.001);

        let a = (out_a.clamp(0.0, 1.0) * 255.0) as u32;
        let r = out_r.clamp(0.0, 255.0) as u32;
        let g = out_g.clamp(0.0, 255.0) as u32;
        let b = out_b.clamp(0.0, 255.0) as u32;

        (a << 24) | (r << 16) | (g << 8) | b
    }
}


