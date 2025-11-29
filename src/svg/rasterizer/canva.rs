use std::collections::HashMap;
use crate::svg::parser::tags::Tag;
use crate::svg::rasterizer::tags::path::draw_path;
use crate::svg::rasterizer::tags::polygon::draw_polygon;
use crate::svg::rasterizer::tags::rect::draw_rect;
use crate::svg::rasterizer::tags::circle::draw_circle;
use crate::svg::rasterizer::tags::ellipse::draw_ellipse;
use crate::svg::rasterizer::tags::line::draw_line;
use crate::svg::rasterizer::tags::polyline::draw_polyline;
use crate::svg::rasterizer::tags::r#use::draw_use;
use crate::svg::rasterizer::tags::text::draw_text;
use crate::svg::utils::transform::{parse_transform, Transform};

pub struct Canvas {
    pub width: usize,
    pub height: usize,
    pub data: Vec<u32>,
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![0xFFFF_FFFF; width * height],
        }
    }

    pub fn new_transparent(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![0x0000_0000; width * height],
        }
    }

    pub fn draw(&mut self, tag: &mut Tag, defs: &HashMap<String, Tag>, transform: &Transform) {

        match &*tag.name {
            "clipPath" | "defs" | "linearGradient" | "radialGradient" |
            "pattern" | "mask" | "marker" | "filter" => return,

            _ => {}
        }

        // Handle Filter
        if let Some(filter_id_raw) = tag.params.get("filter") {
             let filter_id = if filter_id_raw.starts_with("url(#") && filter_id_raw.ends_with(")") {
                 &filter_id_raw[5..filter_id_raw.len()-1]
             } else {
                 filter_id_raw
             };

             if let Some(filter_tag) = defs.get(filter_id) {
                 let mut temp_canvas = Self::new_transparent(self.width, self.height);
                 
                 // Clone tag and remove filter to avoid infinite recursion
                 let mut tag_clone = tag.clone();
                 tag_clone.params.remove("filter");
                 
                 // Draw to temp canvas using the PARENT transform
                 // The draw() method will re-parse the local transform from the tag
                 temp_canvas.draw(&mut tag_clone, defs, transform);
                 
                 let filtered_data = crate::svg::rasterizer::filters::apply_filter(
                     &temp_canvas.data, 
                     self.width, 
                     self.height, 
                     filter_tag, 
                     defs
                 );
                 
                 self.add_buffer(&filtered_data, 0, 0, self.width, self.height);
                 return;
             }
        }

        let local_transform = parse_transform(tag);
        let combined = transform.then(&local_transform);

        match tag.name.as_str() {
            "path" => { draw_path(tag, defs, self, &combined); }
            "rect" => { draw_rect(tag, defs, self, &combined); }
            "polygon" => { draw_polygon(tag, defs, self, &combined); }
            "circle" => { draw_circle(tag, defs, self, &combined); }
            "ellipse" => { draw_ellipse(tag, defs, self, &combined); }
            "line" => { draw_line(tag, defs, self, &combined); }
            "polyline" => { draw_polyline(tag, defs, self, &combined); }
            "use" => { draw_use(tag, defs, self, &combined); }
            "text" => { draw_text(tag, defs, self, &combined); }
            _ => {}
        }

        for child in &mut tag.children {
            self.draw(child, defs, &combined);
        }
    }

     pub fn add_buffer(
        &mut self,
        color_map: &[u32],
        x_offset: isize,
        y_offset: isize,
        colormap_width: usize,
        colormap_height: usize
    ) {
        let buffer_w = colormap_width as isize;
        let buffer_h = colormap_height as isize;

        let canvas_w = self.width as isize;
        let canvas_h = self.height as isize;

        // Calculate intersection of buffer with canvas
        let clip_x1 = x_offset.max(0);
        let clip_y1 = y_offset.max(0);
        let clip_x2 = (x_offset + buffer_w).min(canvas_w);
        let clip_y2 = (y_offset + buffer_h).min(canvas_h);

        // If no intersection, return
        if clip_x1 >= clip_x2 || clip_y1 >= clip_y2 {
            return;
        }

        for screen_y_isize in clip_y1..clip_y2 {
            // Calculate indices for this row
            let src_y = (screen_y_isize - y_offset) as usize;
            let dst_y = screen_y_isize as usize;
            
            let src_row_start = src_y * colormap_width + (clip_x1 - x_offset) as usize;
            let dst_row_start = dst_y * self.width + clip_x1 as usize;
            let len = (clip_x2 - clip_x1) as usize;

            // Get slices for this scanline
            let src_slice = &color_map[src_row_start .. src_row_start + len];
            let dst_slice = &mut self.data[dst_row_start .. dst_row_start + len];
            
            // Blend using SIMD optimized function
            crate::svg::rasterizer::simd::blend_scanline(dst_slice, src_slice);
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


