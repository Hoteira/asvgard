use crate::svg::parser::tags::Tag;

#[inline]
pub fn get_width(tag: &mut Tag) -> usize {
    let w = tag.params.get("width");

    if w.is_none() {
        0
    } else {
        w.unwrap().parse::<usize>().unwrap()
    }
}

#[inline]
pub fn get_height(tag: &mut Tag) -> usize {
    let h = tag.params.get("height");

    if h.is_none() {
        0
    } else {
        h.unwrap().parse::<usize>().unwrap()
    }
}

#[inline]
pub fn get_x(tag: &mut Tag) -> usize {
    let x = tag.params.get("x");

    if x.is_none() {
        0
    } else {
        x.unwrap().parse::<usize>().unwrap()
    }
}

#[inline]
pub fn get_y(tag: &mut Tag) -> usize {
    let y = tag.params.get("y");

    if y.is_none() {
        0
    } else {
        y.unwrap().parse::<usize>().unwrap()
    }
}

pub fn parse_length(s: &str, default: f32, reference: f32) -> f32 {
    if s.trim().ends_with('%') {
        s.trim_end_matches('%').parse::<f32>().unwrap_or(default * 100.0) / 100.0 * reference
    } else {
        s.parse::<f32>().unwrap_or(default)
    }
}
