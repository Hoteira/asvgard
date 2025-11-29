use crate::svg::parser::tags::Tag;

#[inline]
pub fn get_stroke_width(tag: &mut Tag) -> f32 {
    let w = tag.params.get("stroke-width");

    if w.is_none() { 0.0 } else { w.unwrap().parse::<f32>().unwrap_or(0.0) }
}