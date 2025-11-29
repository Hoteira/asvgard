use crate::svg::parser::tags::Tag;
use crate::svg::utils::color::parse_color_value;
use crate::svg::utils::coords::parse_length;

#[derive(Debug, Clone, PartialEq)]
pub enum GradientUnits {
    UserSpaceOnUse,
    ObjectBoundingBox,
}

#[derive(Debug, Clone)]
pub struct LinearGradient {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub stops: Vec<GradientStop>,
    pub units: GradientUnits,
}

#[derive(Debug, Clone)]
pub struct GradientStop {
    pub offset: f32,
    pub color: u32,
}

impl LinearGradient {
    pub fn scale(&mut self, scale: f32) {
        if self.units == GradientUnits::UserSpaceOnUse {
            self.x1 *= scale;
            self.y1 *= scale;
            self.x2 *= scale;
            self.y2 *= scale;
        }
    }

    pub fn interpolate(&self, x: f32, y: f32, bbox_x: f32, bbox_y: f32, bbox_w: f32, bbox_h: f32) -> u32 {
        if self.stops.is_empty() {
            return 0x00000000;
        }

        if self.stops.len() == 1 {
            return self.stops[0].color;
        }

        let (gx1, gy1, gx2, gy2) = if self.units == GradientUnits::ObjectBoundingBox {
            (
                bbox_x + self.x1 * bbox_w,
                bbox_y + self.y1 * bbox_h,
                bbox_x + self.x2 * bbox_w,
                bbox_y + self.y2 * bbox_h,
            )
        } else {
            (self.x1, self.y1, self.x2, self.y2)
        };

        let dx = gx2 - gx1;
        let dy = gy2 - gy1;
        let len_sq = dx * dx + dy * dy;

        if len_sq < 1e-6 {
            return self.stops[0].color;
        }

        let px = x - gx1;
        let py = y - gy1;
        let mut t = (px * dx + py * dy) / len_sq;

        // Basic spread method: pad (clamp)
        // TODO: Implement repeat/reflect
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

pub(crate) fn blend_colors(color1: u32, color2: u32, t: f32) -> u32 {
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
    let x1 = tag.params.get("x1").map(|s| parse_length(s, 0.0, 1.0)).unwrap_or(0.0);
    let y1 = tag.params.get("y1").map(|s| parse_length(s, 0.0, 1.0)).unwrap_or(0.0);
    let x2 = tag.params.get("x2").map(|s| parse_length(s, 1.0, 1.0)).unwrap_or(1.0);
    let y2 = tag.params.get("y2").map(|s| parse_length(s, 0.0, 1.0)).unwrap_or(0.0);

    let units = tag.params.get("gradientUnits")
        .map(|s| s.as_str())
        .unwrap_or("objectBoundingBox");
    
    let units = match units {
        "userSpaceOnUse" => GradientUnits::UserSpaceOnUse,
        _ => GradientUnits::ObjectBoundingBox,
    };

    let mut stops = Vec::new();
    for child in &tag.children {
        if child.name == "stop" {
            let offset = child
                .params
                .get("offset")
                .map(|s| parse_length(s, 0.0, 1.0))
                .unwrap_or(0.0);

            let color = child
                .params
                .get("stop-color")
                .map(|c| parse_color_value(c))
                .unwrap_or(0x000000);

            // stop-opacity support could be added here
            // but we need to blend it into the color alpha
            let opacity = child.params.get("stop-opacity")
                .and_then(|s| s.parse::<f32>().ok())
                .unwrap_or(1.0);

            let final_color = if opacity < 1.0 {
                let a = ((color >> 24) & 0xFF) as f32;
                let new_a = (a * opacity) as u32;
                (color & 0x00FFFFFF) | (new_a << 24)
            } else {
                color
            };

            stops.push(GradientStop { offset, color: final_color });
        }
    }
    
    stops.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap_or(std::cmp::Ordering::Equal));


    LinearGradient {
        x1,
        y1,
        x2,
        y2,
        stops,
        units,
    }
}