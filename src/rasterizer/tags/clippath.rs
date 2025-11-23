use crate::parser::tags::Tag;
use crate::rasterizer::dda::Rasterizer;
use crate::rasterizer::raster::PathRasterizer;
use crate::rasterizer::tags::path::parse_path_data;

pub struct ClipMask {
    pub width: usize,
    pub height: usize,
    pub mask_data: Vec<u8>,
    pub x_offset: f32,
    pub y_offset: f32,
}

impl ClipMask {
    pub fn new(width: usize, height: usize, x_offset: f32, y_offset: f32) -> Self {
        Self {
            width,
            height,
            mask_data: vec![0; width * height],
            x_offset,
            y_offset,
        }
    }

    pub fn from_path( tag: &Tag, scale: f32 ) -> Option<Self> {
        let d = tag.params.get("d")?;
        let d_path = parse_path_data(d);

        let mut path_rasterizer = PathRasterizer::new();
        path_rasterizer.build_lines_from_path(&d_path, scale, 1.0);

        let width = path_rasterizer.bounds.width.ceil() as usize;
        let height = path_rasterizer.bounds.height.ceil() as usize;

        if width == 0 || height == 0 {
            return None;
        }

        let renderer = Rasterizer::new(width, height);
        let mask_data = renderer
            .draw(&path_rasterizer.v_lines, &path_rasterizer.m_lines)
            .to_bitmap();

        Some(ClipMask {
            width,
            height,
            mask_data,
            x_offset: path_rasterizer.bounds.x,
            y_offset: path_rasterizer.bounds.y,
        })
    }

    pub fn from_rect(
        tag: &Tag,
        scale: f32,
    ) -> Option<Self> {
        let x = tag.params.get("x")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.0);
        let y = tag.params.get("y")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.0);
        let width = tag.params.get("width")
            .and_then(|s| s.parse::<f32>().ok())?;
        let height = tag.params.get("height")
            .and_then(|s| s.parse::<f32>().ok())?;

        let scaled_width = (width * scale).ceil() as usize;
        let scaled_height = (height * scale).ceil() as usize;

        if scaled_width == 0 || scaled_height == 0 {
            return None;
        }

        let mask_data = vec![255u8; scaled_width * scaled_height];

        Some(ClipMask {
            width: scaled_width,
            height: scaled_height,
            mask_data,
            x_offset: x * scale,
            y_offset: y * scale,
        })
    }

    pub fn from_clippath_tag(
        clippath_tag: &Tag,
        scale: f32,
    ) -> Option<Self> {
        let mut combined_mask: Option<ClipMask> = None;

        for child in &clippath_tag.children {
            let child_mask = match child.name.as_str() {
                "path" => Self::from_path(child, scale),
                "rect" => Self::from_rect(child, scale),
                "circle" => Self::from_circle(child, scale),
                _ => continue,
            };

            if let Some(child_mask) = child_mask {
                if let Some(ref mut existing) = combined_mask {
                    existing.union_with(&child_mask);
                } else {
                    combined_mask = Some(child_mask);
                }
            }
        }

        combined_mask
    }

    pub fn from_circle(
        tag: &Tag,
        scale: f32,
    ) -> Option<Self> {
        let cx = tag.params.get("cx")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.0);
        let cy = tag.params.get("cy")
            .and_then(|s| s.parse::<f32>().ok())
            .unwrap_or(0.0);
        let r = tag.params.get("r")
            .and_then(|s| s.parse::<f32>().ok())?;

        let scaled_r = r * scale.max(scale);
        let width = (scaled_r * 2.0).ceil() as usize;
        let height = width;

        if width == 0 {
            return None;
        }

        let mut mask_data = vec![0u8; width * height];
        let center_x = scaled_r;
        let center_y = scaled_r;

        for y in 0..height {
            for x in 0..width {
                let dx = x as f32 - center_x;
                let dy = y as f32 - center_y;
                let dist = (dx * dx + dy * dy).sqrt();

                if dist <= scaled_r {
                    mask_data[y * width + x] = 255;
                }
            }
        }

        Some(ClipMask {
            width,
            height,
            mask_data,
            x_offset: (cx - r) * scale,
            y_offset: (cy - r) * scale,
        })
    }

    pub fn union_with(&mut self, other: &ClipMask) {

        let min_x = self.x_offset.min(other.x_offset);
        let min_y = self.y_offset.min(other.y_offset);
        let max_x = (self.x_offset + self.width as f32)
            .max(other.x_offset + other.width as f32);
        let max_y = (self.y_offset + self.height as f32)
            .max(other.y_offset + other.height as f32);

        let new_width = (max_x - min_x).ceil() as usize;
        let new_height = (max_y - min_y).ceil() as usize;
        let mut new_mask = vec![0u8; new_width * new_height];

        for y in 0..self.height {
            for x in 0..self.width {
                let src_idx = y * self.width + x;
                let dst_x = ((self.x_offset - min_x) + x as f32) as usize;
                let dst_y = ((self.y_offset - min_y) + y as f32) as usize;
                let dst_idx = dst_y * new_width + dst_x;
                new_mask[dst_idx] = new_mask[dst_idx].max(self.mask_data[src_idx]);
            }
        }

        for y in 0..other.height {
            for x in 0..other.width {
                let src_idx = y * other.width + x;
                let dst_x = ((other.x_offset - min_x) + x as f32) as usize;
                let dst_y = ((other.y_offset - min_y) + y as f32) as usize;
                let dst_idx = dst_y * new_width + dst_x;
                new_mask[dst_idx] = new_mask[dst_idx].max(other.mask_data[src_idx]);
            }
        }

        self.width = new_width;
        self.height = new_height;
        self.mask_data = new_mask;
        self.x_offset = min_x;
        self.y_offset = min_y;
    }

    pub fn apply_to_bitmap(&self, bitmap: &mut [u32], bitmap_x: f32, bitmap_y: f32, bitmap_width: usize, bitmap_height: usize) {
        for y in 0..bitmap_height {
            for x in 0..bitmap_width {
                let world_x = bitmap_x + x as f32;
                let world_y = bitmap_y + y as f32;

                let mask_x = (world_x - self.x_offset) as i32;
                let mask_y = (world_y - self.y_offset) as i32;

                let coverage = if mask_x >= 0 && mask_x < self.width as i32 &&
                    mask_y >= 0 && mask_y < self.height as i32 {
                    let mask_idx = mask_y as usize * self.width + mask_x as usize;
                    self.mask_data[mask_idx]
                } else {
                    0
                };

                let bitmap_idx = y * bitmap_width + x;
                let current_alpha = ((bitmap[bitmap_idx] >> 24) & 0xFF) as u8;

                let new_alpha = ((current_alpha as u32 * coverage as u32) / 255) as u8;
                bitmap[bitmap_idx] = (bitmap[bitmap_idx] & 0x00FFFFFF) | ((new_alpha as u32) << 24);
            }
        }
    }
}

pub fn get_clip_path_id(tag: &Tag) -> Option<String> {
    tag.params.get("clip-path").and_then(|cp| {
        let cp = cp.trim();
        if cp.starts_with("url(#") && cp.ends_with(")") {
            Some(cp[5..cp.len()-1].to_string())
        } else {
            None
        }
    })
}