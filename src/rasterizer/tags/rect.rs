use std::collections::HashMap;
use crate::parser::tags::Tag;
use crate::rasterizer::canva::Canvas;
use crate::utils::color::{get_fill, get_stroke, Paint};
use crate::utils::coords::{get_height, get_width, get_x, get_y};

pub fn draw_rect(tag: &mut Tag, defs: &HashMap<String, Tag>, canvas: &mut Canvas, scale: f32, offset_x: f32, offset_y: f32) {
    let width = get_width(tag);
    let height = get_height(tag);
    let x = get_x(tag);
    let y = get_y(tag);

    let scaled_x = (x as f32 * scale + offset_x) as usize;
    let scaled_y = (y as f32 * scale + offset_y) as usize;
    let scaled_width = (width as f32 * scale) as usize;
    let scaled_height = (height as f32 * scale) as usize;

    let fill = get_fill(tag).resolve(defs);
    let stroke = get_stroke(tag).resolve(defs);

    if !fill.is_none() {
        for i in 0..scaled_height {
            for j in 0..scaled_width {
                let px = j + scaled_x;
                let py = i + scaled_y;
                if px < canvas.width && py < canvas.height {

                    let svg_x = x as f32 + (j as f32 / scale);
                    let svg_y = y as f32 + (i as f32 / scale);
                    let color = fill.get_color_at(svg_x, svg_y);
                    canvas.data[py * canvas.width + px] = color;
                }
            }
        }
    }

    // TODO: Draw stroke
}