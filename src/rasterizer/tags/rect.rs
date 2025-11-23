
use std::collections::HashMap;
use crate::parser::tags::Tag;
use crate::rasterizer::canva::Canvas;
use crate::rasterizer::map::Map;
use crate::utils::color::{get_fill, get_stroke, Paint};
use crate::utils::transform::Transform;

pub fn draw_rect(
    tag: &mut Tag,
    defs: &HashMap<String, Tag>,
    canvas: &mut Map,
    transform: &Transform,
) {}