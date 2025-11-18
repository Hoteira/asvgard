use std::collections::HashMap;
use crate::parser::tags::Tag;
use crate::utils::color::{parse_color_value, Paint};

#[derive(Debug, Clone)]
pub struct LinearGradient {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub stops: Vec<GradientStop>,
}

#[derive(Debug, Clone)]
pub struct GradientStop {
    pub offset: f32,
    pub color: u32,
}

impl LinearGradient {

    pub fn scale(&mut self, scale_x: f32, scale_y: f32) {
        self.x1 *= scale_x;
        self.y1 *= scale_y;
        self.x2 *= scale_x;
        self.y2 *= scale_y;
    }

    pub fn interpolate(&self, x: f32, y: f32) -> u32 {
        if self.stops.is_empty() {
            return 0x00000000;
        }

        if self.stops.len() == 1 {
            return self.stops[0].color;
        }

        let dx = self.x2 - self.x1;
        let dy = self.y2 - self.y1;
        let len_sq = dx * dx + dy * dy;

        if len_sq < 1e-6 {
            return self.stops[0].color;
        }

        let px = x - self.x1;
        let py = y - self.y1;
        let mut t = (px * dx + py * dy) / len_sq;

        t = t.clamp(0.0, 1.0);


        if t <= self.stops[0].offset {
            return self.stops[0].color;
        }
        if t >= self.stops[self.stops.len() - 1].offset {
            return self.stops[self.stops.len() - 1].color;
        }

        for i in 0..self.stops.len() - 1 {
            let stop1 = &self.stops[i];
            let stop2 = &self.stops[i + 1];

            if t >= stop1.offset && t <= stop2.offset {
                let range = stop2.offset - stop1.offset;
                let local_t = if range > 1e-6 {
                    (t - stop1.offset) / range
                } else {
                    0.0
                };

                return blend_colors(stop1.color, stop2.color, local_t);
            }
        }

        self.stops[self.stops.len() - 1].color
    }
}

fn blend_colors(color1: u32, color2: u32, t: f32) -> u32 {
    let a1 = ((color1 >> 24) & 0xFF) as f32;
    let r1 = ((color1 >> 16) & 0xFF) as f32;
    let g1 = ((color1 >> 8) & 0xFF) as f32;
    let b1 = (color1 & 0xFF) as f32;

    let a2 = ((color2 >> 24) & 0xFF) as f32;
    let r2 = ((color2 >> 16) & 0xFF) as f32;
    let g2 = ((color2 >> 8) & 0xFF) as f32;
    let b2 = (color2 & 0xFF) as f32;

    let a = (a1 + (a2 - a1) * t) as u32;
    let r = (r1 + (r2 - r1) * t) as u32;
    let g = (g1 + (g2 - g1) * t) as u32;
    let b = (b1 + (b2 - b1) * t) as u32;

    (a << 24) | (r << 16) | (g << 8) | b
}

pub fn load_linear_gradient(tag: &Tag) -> LinearGradient {
    let x1 = tag.params.get("x1")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0);
    let y1 = tag.params.get("y1")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0);
    let x2 = tag.params.get("x2")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0);
    let y2 = tag.params.get("y2")
        .and_then(|s| s.parse::<f32>().ok())
        .unwrap_or(0.0);

    let mut stops = Vec::new();
    for child in &tag.children {
        if child.name == "stop" {
            let offset = child.params.get("offset")
                .and_then(|s| s.trim_end_matches('%').parse::<f32>().ok())
                .unwrap_or(0.0);

            let color = child.params.get("stop-color")
                .map(|c| parse_color_value(c))
                .unwrap_or(0x000000);

            stops.push(GradientStop { offset, color });
        } else {
            panic!("{}", child.name);
        }
    }

    println!("Linear gradient: x1={}, y1={}, x2={}, y2={}, stops={:?}", x1, y1, x2, y2, stops);

    LinearGradient { x1, y1, x2, y2, stops }
}