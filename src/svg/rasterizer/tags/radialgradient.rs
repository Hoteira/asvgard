use crate::svg::parser::tags::Tag;
use crate::svg::rasterizer::tags::lineargradient::{GradientStop, GradientUnits, blend_colors};
use crate::svg::utils::color::parse_color_value;
use crate::svg::utils::coords::parse_length;
use crate::utils::compat::FloatExt;
use crate::utils::compat::Vec;
use core::cmp::Ordering;

#[derive(Debug, Clone)]
pub struct RadialGradient {
    pub cx: f32,
    pub cy: f32,
    pub r: f32,
    pub fx: f32,
    pub fy: f32,
    pub stops: Vec<GradientStop>,
    pub units: GradientUnits,
}

impl RadialGradient {
    pub fn interpolate(&self, x: f32, y: f32, bbox_x: f32, bbox_y: f32, bbox_w: f32, bbox_h: f32) -> u32 {
        if self.stops.is_empty() {
            return 0x00000000;
        }

        if self.stops.len() == 1 {
            return self.stops[0].color;
        }

        // Resolve coordinates based on units
        let (cx, cy, r, fx, fy) = if self.units == GradientUnits::ObjectBoundingBox {
            (
                bbox_x + self.cx * bbox_w,
                bbox_y + self.cy * bbox_h,
                self.r * (bbox_w.powi(2) + bbox_h.powi(2)).sqrt() / 2.0f32.sqrt(),
                bbox_x + self.fx * bbox_w,
                bbox_y + self.fy * bbox_h
            )
        } else {
            (self.cx, self.cy, self.r, self.fx, self.fy)
        };

        let t = if self.units == GradientUnits::ObjectBoundingBox {
            // Transform P to unit space
            let nx = if bbox_w > 1e-6 { (x - bbox_x) / bbox_w } else { 0.0 };
            let ny = if bbox_h > 1e-6 { (y - bbox_y) / bbox_h } else { 0.0 };
            
            calculate_t(nx, ny, self.cx, self.cy, self.r, self.fx, self.fy)
        } else {
            calculate_t(x, y, cx, cy, r, fx, fy)
        };

        let t = t.clamp(0.0, 1.0);

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

fn calculate_t(x: f32, y: f32, cx: f32, cy: f32, r: f32, fx: f32, fy: f32) -> f32 {
    // V = C - F
    let vx = cx - fx;
    let vy = cy - fy;
    
    // D = P - F
    let dx = x - fx;
    let dy = y - fy;

    // A = V.V - R^2
    // B = -2 (D.V)
    // C = D.D

    let r2 = r * r;
    let v_dot_v = vx * vx + vy * vy;
    let d_dot_v = dx * vx + dy * vy;
    let d_dot_d = dx * dx + dy * dy;

    let a = v_dot_v - r2;
    let b = -2.0 * d_dot_v;
    let c = d_dot_d;

    if a.abs() < 1e-6 {
        // Linear equation B t + C = 0 => t = -C / B
        if b.abs() < 1e-6 {
            return 0.0; // Degenerate
        }
        return -c / b;
    }

    let discriminant = b * b - 4.0 * a * c;
    if discriminant < 0.0 {
        return 0.0; // Should not happen if P is inside? Or maybe outside. 
        // If outside cone, t is undefined or we clamp.
        // We treat it as 1.0 (outside) or 0.0?
        // Usually we clamp t to 1.0 if outside.
        // But let's return 0.0 for now.
    }

    let sqrt_disc = discriminant.sqrt();
    let t1 = (-b + sqrt_disc) / (2.0 * a);
    let t2 = (-b - sqrt_disc) / (2.0 * a);

    // We want the positive t > 0.
    // Actually, t represents the scaling factor of the radius.
    // We usually want the smallest positive solution that makes sense?
    // No, for radial gradient, t maps 0 (at F) to 1 (at Circle).
    // So we expect 0 <= t <= 1 inside the gradient area.
    // If point is outside, t > 1.
    
    t1.max(t2)
}

pub fn load_radial_gradient(tag: &Tag) -> RadialGradient {
    // Defaults: cx=50%, cy=50%, r=50%
    let cx = tag.params.get("cx").map(|s| parse_length(s, 0.5, 1.0)).unwrap_or(0.5);
    let cy = tag.params.get("cy").map(|s| parse_length(s, 0.5, 1.0)).unwrap_or(0.5);
    let r = tag.params.get("r").map(|s| parse_length(s, 0.5, 1.0)).unwrap_or(0.5);
    
    let fx = tag.params.get("fx").map(|s| parse_length(s, cx, 1.0)).unwrap_or(cx);
    let fy = tag.params.get("fy").map(|s| parse_length(s, cy, 1.0)).unwrap_or(cy);

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

            // stop-opacity support
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
    
    stops.sort_by(|a, b| a.offset.partial_cmp(&b.offset).unwrap_or(Ordering::Equal));

    RadialGradient {
        cx, cy, r, fx, fy, stops, units
    }
}
