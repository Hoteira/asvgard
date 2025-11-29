use crate::svg::parser::tags::Tag;
use crate::utils::compat::HashMap;
use crate::svg::rasterizer::tags::lineargradient::{LinearGradient, load_linear_gradient};
use crate::svg::rasterizer::tags::radialgradient::{RadialGradient, load_radial_gradient};
use crate::utils::compat::{String, ToString, Vec};

#[derive(Debug, Clone)]
pub enum Paint {
    Solid(u32),
    LinearGradient(LinearGradient),
    RadialGradient(RadialGradient),
    Reference(String),
    None,
}

impl Paint {
    pub fn scale(&mut self, scale: f32) {
        match self {
            Paint::LinearGradient(gradient) => gradient.scale(scale),
            Paint::RadialGradient(_) => {},
            _ => {}
        }
    }
    pub fn resolve(&self, defs: &HashMap<String, Tag>) -> Paint {
        match self {
            Paint::Reference(id) => {

                if let Some(tag) = defs.get(id) {
                    match tag.name.as_str() {
                        "linearGradient" => Paint::LinearGradient(
                            load_linear_gradient(tag),
                        ),
                        "radialGradient" => Paint::RadialGradient(
                             load_radial_gradient(tag),
                        ),
                        _ => {
                            #[cfg(feature = "std")]
                            std::println!("ID {} found but tag name is {}", id, tag.name);
                            Paint::None
                        },
                    }
                } else {
                    #[cfg(feature = "std")]
                    std::println!("ID not found in defs: '{}'", id);
                    Paint::None
                }
            }
            _ => self.clone(),
        }
    }

    pub fn get_color_at(&self, x: f32, y: f32, bbox_x: f32, bbox_y: f32, bbox_w: f32, bbox_h: f32) -> u32 {
        match self {
            Paint::Solid(color) => *color,
            Paint::LinearGradient(gradient) => gradient.interpolate(x, y, bbox_x, bbox_y, bbox_w, bbox_h),
            Paint::RadialGradient(gradient) => gradient.interpolate(x, y, bbox_x, bbox_y, bbox_w, bbox_h),
            Paint::None | Paint::Reference(_) => 0x00000000,
        }
    }

    pub fn is_none(&self) -> bool {
        matches!(self, Paint::None)
    }
}

#[inline]
pub fn get_fill(tag: &Tag) -> Paint {
    tag.params
        .get("fill")
        .map(|c| parse_paint(c))
        .unwrap_or(Paint::Solid(0x0000_0000))
}

#[inline]
pub fn get_stroke(tag: &Tag) -> Paint {
    tag.params
        .get("stroke")
        .map(|c| parse_paint(c))
        .unwrap_or(Paint::None)
}

fn parse_paint(s: &str) -> Paint {
    let s = s.trim();

    if s.eq_ignore_ascii_case("none") {
        return Paint::None;
    }

    if s.starts_with("url(#") {
        let id = s.trim_start_matches("url(#").trim_end_matches(')');
        return Paint::Reference(id.to_string());
    }

    Paint::Solid(parse_color_value(&s))
}

pub(crate) fn parse_color_value(c: &str) -> u32 {
    let c = c.trim().to_lowercase();

    if let Some(named_color) = named(&c) {
        return named_color;
    }

    if c.starts_with("#") {
        return parse_hex(&c);
    }

    if c.starts_with("rgb(") || c.starts_with("rgba(") {
        return parse_rgb(&c);
    }

    if c.starts_with("hsl(") || c.starts_with("hsla(") {
        return parse_hsl(&c);
    }

            0x0000_0000}

fn parse_hsl(hsl: &str) -> u32 {
    let is_hsla = hsl.starts_with("hsla(");
    let inner = hsl
        .trim_start_matches("hsl(")
        .trim_start_matches("hsla(")
        .trim_end_matches(')');

    let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();

    if parts.len() < 3 {
        return 0xFF000000;
    }

    let h = parts[0].parse::<f32>().ok().unwrap_or(0.0);
    let s = parts[1]
        .trim_end_matches('%')
        .parse::<f32>()
        .ok()
        .unwrap_or(0.0)
        / 100.0;
    let l = parts[2]
        .trim_end_matches('%')
        .parse::<f32>()
        .ok()
        .unwrap_or(0.0)
        / 100.0;

    let a = if is_hsla && parts.len() >= 4 {
        parse_alpha(parts[3])
    } else {
        255
    };

    let (r, g, b) = hsl_to_rgb(h, s, l);

    (a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | (b as u32)
}

fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (u8, u8, u8) {
    let h = (h % 360.0) / 360.0;

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };

    let p = 2.0 * l - q;

    let r = hue_to_rgb(p, q, h + 1.0 / 3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0 / 3.0);

    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 {
        t += 1.0;
    }
    if t > 1.0 {
        t -= 1.0;
    }

    if t < 1.0 / 6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 1.0 / 2.0 {
        q
    } else if t < 2.0 / 3.0 {
        p + (q - p) * (2.0 / 3.0 - t) * 6.0
    } else {
        p
    }
}

fn parse_rgb(rgb: &str) -> u32 {
    let is_rgba = rgb.starts_with("rgba(");
    let inner = rgb
        .trim_start_matches("rgb(")
        .trim_start_matches("rgba(")
        .trim_end_matches(')');

    let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();

    if parts.len() < 3 {
        return 0xFF000000;
    }

    let r = parse_color_component(parts[0]);
    let g = parse_color_component(parts[1]);
    let b = parse_color_component(parts[2]);

    let a = if is_rgba && parts.len() >= 4 {
        parse_alpha(parts[3])
    } else {
        255
    };

    (a as u32) << 24 | (r as u32) << 16 | (g as u32) << 8 | (b as u32)
}

fn parse_color_component(s: &str) -> u8 {
    if s.ends_with('%') {
        let percent = s.trim_end_matches('%').parse::<f32>().ok().unwrap_or(0.0);
        (percent * 2.55).clamp(0.0, 255.0) as u8
    } else {
        s.parse::<u8>().ok().unwrap_or(0)
    }
}

fn parse_alpha(s: &str) -> u8 {
    if s.ends_with('%') {
        let percent = s.trim_end_matches('%').parse::<f32>().ok().unwrap_or(0.0);
        (percent * 2.55).clamp(0.0, 255.0) as u8
    } else {
        let alpha = s.parse::<f32>().ok().unwrap_or(0.0);
        (alpha * 255.0).clamp(0.0, 255.0) as u8
    }
}

fn parse_hex(hex: &str) -> u32 {
    let hex = hex.trim_start_matches('#');

    match hex.len() {
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok().unwrap_or(0);
            let g = u8::from_str_radix(&hex[1..2], 16).ok().unwrap_or(0);
            let b = u8::from_str_radix(&hex[2..3], 16).ok().unwrap_or(0);
            0xFF000000 | ((r as u32 * 17) << 16) | ((g as u32 * 17) << 8) | (b as u32 * 17)
        }
        6 => {
            let rgb = u32::from_str_radix(hex, 16).ok().unwrap_or(0);
            0xFF000000 | rgb
        }
        8 => u32::from_str_radix(hex, 16).ok().unwrap_or(0xFF000000),
        _ => 0xFF000000,
    }
}

pub fn named(str: &str) -> Option<u32> {
    match str {
        "aliceblue" => Some(0xFFF0F8FF),
        "antiquewhite" => Some(0xFFFAEBD7),
        "aqua" => Some(0xFF00FFFF),
        "aquamarine" => Some(0xFF7FFFD4),
        "azure" => Some(0xFFF0FFFF),
        "beige" => Some(0xFFF5F5DC),
        "bisque" => Some(0xFFFFE4C4),
        "black" => Some(0xFF000000),
        "blanchedalmond" => Some(0xFFFFEBCD),
        "blue" => Some(0xFF0000FF),
        "blueviolet" => Some(0xFF8A2BE2),
        "brown" => Some(0xFFA52A2A),
        "burlywood" => Some(0xFFDEB887),
        "cadetblue" => Some(0xFF5F9EA0),
        "chartreuse" => Some(0xFF7FFF00),
        "chocolate" => Some(0xFFD2691E),
        "coral" => Some(0xFFFF7F50),
        "cornflowerblue" => Some(0xFF6495ED),
        "cornsilk" => Some(0xFFFFF8DC),
        "crimson" => Some(0xFFDC143C),
        "cyan" => Some(0xFF00FFFF),
        "darkblue" => Some(0xFF00008B),
        "darkcyan" => Some(0xFF008B8B),
        "darkgoldenrod" => Some(0xFFB8860B),
        "darkgray" => Some(0xFFA9A9A9),
        "darkgrey" => Some(0xFFA9A9A9),
        "darkgreen" => Some(0xFF006400),
        "darkkhaki" => Some(0xFFBDB76B),
        "darkmagenta" => Some(0xFF8B008B),
        "darkolivegreen" => Some(0xFF556B2F),
        "darkorange" => Some(0xFFFF8C00),
        "darkorchid" => Some(0xFF9932CC),
        "darkred" => Some(0xFF8B0000),
        "darksalmon" => Some(0xFFE9967A),
        "darkseagreen" => Some(0xFF8FBC8F),
        "darkslateblue" => Some(0xFF483D8B),
        "darkslategray" => Some(0xFF2F4F4F),
        "darkslategrey" => Some(0xFF2F4F4F),
        "darkturquoise" => Some(0xFF00CED1),
        "darkviolet" => Some(0xFF9400D3),
        "deeppink" => Some(0xFFFF1493),
        "deepskyblue" => Some(0xFF00BFFF),
        "dimgray" => Some(0xFF696969),
        "dimgrey" => Some(0xFF696969),
        "dodgerblue" => Some(0xFF1E90FF),
        "firebrick" => Some(0xFFB22222),
        "floralwhite" => Some(0xFFFFFAF0),
        "forestgreen" => Some(0xFF228B22),
        "fuchsia" => Some(0xFFFF00FF),
        "gainsboro" => Some(0xFFDCDCDC),
        "ghostwhite" => Some(0xFFF8F8FF),
        "gold" => Some(0xFFFFD700),
        "goldenrod" => Some(0xFFDAA520),
        "gray" => Some(0xFF808080),
        "grey" => Some(0xFF808080),
        "green" => Some(0xFF008000),
        "greenyellow" => Some(0xFFADFF2F),
        "honeydew" => Some(0xFFF0FFF0),
        "hotpink" => Some(0xFFFF69B4),
        "indianred" => Some(0xFFCD5C5C),
        "indigo" => Some(0xFF4B0082),
        "ivory" => Some(0xFFFFFFF0),
        "khaki" => Some(0xFFF0E68C),
        "lavender" => Some(0xFFE6E6FA),
        "lavenderblush" => Some(0xFFFFF0F5),
        "lawngreen" => Some(0xFF7CFC00),
        "lemonchiffon" => Some(0xFFFFFACD),
        "lightblue" => Some(0xFFADD8E6),
        "lightcoral" => Some(0xFFF08080),
        "lightcyan" => Some(0xFFE0FFFF),
        "lightgoldenrodyellow" => Some(0xFFFAFAD2),
        "lightgray" => Some(0xFFD3D3D3),
        "lightgrey" => Some(0xFFD3D3D3),
        "lightgreen" => Some(0xFF90EE90),
        "lightpink" => Some(0xFFFFB6C1),
        "lightsalmon" => Some(0xFFFFA07A),
        "lightseagreen" => Some(0xFF20B2AA),
        "lightskyblue" => Some(0xFF87CEFA),
        "lightslategray" => Some(0xFF778899),
        "lightslategrey" => Some(0xFF778899),
        "lightsteelblue" => Some(0xFFB0C4DE),
        "lightyellow" => Some(0xFFFFFFE0),
        "lime" => Some(0xFF00FF00),
        "limegreen" => Some(0xFF32CD32),
        "linen" => Some(0xFFFAF0E6),
        "magenta" => Some(0xFFFF00FF),
        "maroon" => Some(0xFF800000),
        "mediumaquamarine" => Some(0xFF66CDAA),
        "mediumblue" => Some(0xFF0000CD),
        "mediumorchid" => Some(0xFFBA55D3),
        "mediumpurple" => Some(0xFF9370DB),
        "mediumseagreen" => Some(0xFF3CB371),
        "mediumslateblue" => Some(0xFF7B68EE),
        "mediumspringgreen" => Some(0xFF00FA9A),
        "mediumturquoise" => Some(0xFF48D1CC),
        "mediumvioletred" => Some(0xFFC71585),
        "midnightblue" => Some(0xFF191970),
        "mintcream" => Some(0xFFF5FFFA),
        "mistyrose" => Some(0xFFFFE4E1),
        "moccasin" => Some(0xFFFFE4B5),
        "navajowhite" => Some(0xFFFFDEAD),
        "navy" => Some(0xFF000080),
        "oldlace" => Some(0xFFFDF5E6),
        "olive" => Some(0xFF808000),
        "olivedrab" => Some(0xFF6B8E23),
        "orange" => Some(0xFFFFA500),
        "orangered" => Some(0xFFFF4500),
        "orchid" => Some(0xFFDA70D6),
        "palegoldenrod" => Some(0xFFEEE8AA),
        "palegreen" => Some(0xFF98FB98),
        "paleturquoise" => Some(0xFFAFEEEE),
        "palevioletred" => Some(0xFFDB7093),
        "papayawhip" => Some(0xFFFFEFD5),
        "peachpuff" => Some(0xFFFFDAB9),
        "peru" => Some(0xFFCD853F),
        "pink" => Some(0xFFFFC0CB),
        "plum" => Some(0xFFDDA0DD),
        "powderblue" => Some(0xFFB0E0E6),
        "purple" => Some(0xFF800080),
        "rebeccapurple" => Some(0xFF663399),
        "red" => Some(0xFFFF0000),
        "rosybrown" => Some(0xFFBC8F8F),
        "royalblue" => Some(0xFF4169E1),
        "saddlebrown" => Some(0xFF8B4513),
        "salmon" => Some(0xFFFA8072),
        "sandybrown" => Some(0xFFF4A460),
        "seagreen" => Some(0xFF2E8B57),
        "seashell" => Some(0xFFFFF5EE),
        "sienna" => Some(0xFFA0522D),
        "silver" => Some(0xFFC0C0C0),
        "skyblue" => Some(0xFF87CEEB),
        "slateblue" => Some(0xFF6A5ACD),
        "slategray" => Some(0xFF708090),
        "slategrey" => Some(0xFF708090),
        "snow" => Some(0xFFFFFAF0),
        "springgreen" => Some(0xFF00FF7F),
        "steelblue" => Some(0xFF4682B4),
        "tan" => Some(0xFFD2B48C),
        "teal" => Some(0xFF008080),
        "thistle" => Some(0xFFD8BFD8),
        "tomato" => Some(0xFFFF6347),
        "turquoise" => Some(0xFF40E0D0),
        "violet" => Some(0xFFEE82EE),
        "wheat" => Some(0xFFF5DEB3),
        "white" => Some(0xFFFFFFFF),
        "whitesmoke" => Some(0xFFF5F5F5),
        "yellow" => Some(0xFFFFFF00),
        "yellowgreen" => Some(0xFF9ACD32),
        "transparent" => Some(0x00000000),
        "none" => Some(0x00000000),
        _ => None,
    }
}
