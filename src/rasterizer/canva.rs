use std::collections::HashMap;
use crate::parser::tags::Tag;
use crate::rasterizer::tags::path::draw_path;
use crate::rasterizer::tags::polygon::draw_polygon;
use crate::rasterizer::tags::rect::draw_rect;
use crate::rasterizer::map::Map;
use crate::utils::get_id;
use crate::utils::transform::{parse_transform, Transform};

pub struct Canvas {
    pub(crate) width: usize,
    pub(crate) height: usize,
    pub(crate) data: Vec<u32>,

    pub map: Map
}

impl Canvas {
    pub fn new(width: usize, height: usize) -> Self {
        Self {
            width,
            height,
            data: vec![0xFF00_0000; width * height],
            map: Map::new(width, height),
        }
    }
}

impl Canvas {

    pub fn draw(&mut self, tag: &mut Tag, defs: &HashMap<String, Tag>, transform: &Transform) {
        self.map.draw(tag, defs, transform);
    }

    pub fn render(&mut self) {
        for i in 0..self.map.pixels.len() {
            self.data[i] = self.map.pixels[i].to_color();
        }
    }
}


