use crate::parser::tags::Tag;
use crate::utils::color::get_color;

pub struct LinearGradient {
    stops: Vec<((u32, String), f32)>,
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

pub fn load_l_gradient(tag: &Tag) -> LinearGradient {
    let x1 = get_x1(tag).unwrap_or(0.0);
    let y1 = get_y1(tag).unwrap_or(0.0);
    let x2 = get_x2(tag).unwrap_or(0.0);
    let y2 = get_y2(tag).unwrap_or(0.0);
    let mut stops = Vec::new();

    for child in &tag.children {
        if child.name == "stop" {
            let c = get_stop_color(child).unwrap_or((0, String::from("")));
            let o = get_offset(child).unwrap_or(0.0);
            stops.push((c, o));
        }
    }

    LinearGradient {
        x1,
        x2,
        y1,
        y2,
        stops: stops.clone()
    }
}

pub fn get_x1(tag: &Tag) -> Option<f32> {
    Some(tag.params.get("x1").unwrap_or(&String::from("")).parse::<f32>().unwrap_or(0.0))
}

pub fn get_y1(tag: &Tag) -> Option<f32> {
    Some(tag.params.get("y1").unwrap_or(&String::from("")).parse::<f32>().unwrap_or(0.0))
}

pub fn get_x2(tag: &Tag) -> Option<f32> {
    Some(tag.params.get("x2").unwrap_or(&String::from("")).parse::<f32>().unwrap_or(0.0))
}

pub fn get_y2(tag: &Tag) -> Option<f32> {
    Some(tag.params.get("y2").unwrap_or(&String::from("")).parse::<f32>().unwrap_or(0.0))
}

pub fn get_offset(tag: &Tag) -> Option<f32> {
    Some(tag.params.get("offset").unwrap_or(&String::from("")).parse::<f32>().unwrap_or(0.0))
}

pub fn get_stop_color(tag: &Tag) -> Option<(u32, String)> {
    let color = tag.params.get("stop-color");
    Some(get_color(color))
}