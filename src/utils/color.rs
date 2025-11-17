use crate::parser::tags::Tag;

#[inline]
pub fn get_fill(tag: &mut Tag) -> u32 {
    let c = tag.params.get("fill");

    get_color(c)
}

#[inline]
pub fn get_stroke(tag: &mut Tag) -> u32 {
    let c = tag.params.get("stroke");

    get_color(c)
}


#[inline]
pub fn get_color(c: Option<&String>) -> u32 {

    if c.is_some() {
        let c = c.unwrap().trim().to_lowercase();

        if named(&c).is_some() {
            return named(&c).unwrap();
        }

        return parse_color(&c);
    }

    0x0
}

pub fn parse_color(c: &str) -> u32 {

    if c.starts_with("#") {
        return parse_hex(c);
    }

    if c.starts_with("rgb(") || c.starts_with("rgba(") {
        return parse_rgb(c);
    }

    if c.starts_with("hsl(") || c.starts_with("hsla(") {
        return parse_hsl(c);

    }

    0
}

fn parse_hsl(hsl: &str) -> u32 {
    let is_hsla = hsl.starts_with("hsla(");
    let inner = hsl.trim_start_matches("hsl(")
        .trim_start_matches("hsla(")
        .trim_end_matches(')');

    let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();

    if parts.len() < 3 {
        return 0x0;
    }

    let h = parts[0].parse::<f32>().ok().unwrap_or(0.0);
    let s = parts[1].trim_end_matches('%').parse::<f32>().ok().unwrap_or(0.0) / 100.0;
    let l = parts[2].trim_end_matches('%').parse::<f32>().ok().unwrap_or(0.0) / 100.0;

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

    let r = hue_to_rgb(p, q, h + 1.0/3.0);
    let g = hue_to_rgb(p, q, h);
    let b = hue_to_rgb(p, q, h - 1.0/3.0);

    ((r * 255.0) as u8, (g * 255.0) as u8, (b * 255.0) as u8)
}

fn hue_to_rgb(p: f32, q: f32, mut t: f32) -> f32 {
    if t < 0.0 { t += 1.0; }
    if t > 1.0 { t -= 1.0; }

    if t < 1.0/6.0 {
        p + (q - p) * 6.0 * t
    } else if t < 1.0/2.0 {
        q
    } else if t < 2.0/3.0 {
        p + (q - p) * (2.0/3.0 - t) * 6.0
    } else {
        p
    }
}

fn parse_rgb(rgb: &str) -> u32 {
    let is_rgba = rgb.starts_with("rgba(");
    let inner = rgb.trim_start_matches("rgb(")
        .trim_start_matches("rgba(")
        .trim_end_matches(')');

    let parts: Vec<&str> = inner.split(',').map(|s| s.trim()).collect();

    if parts.len() < 3 {
        return 0x0;
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
        // #RGB -> #RRGGBB
        3 => {
            let r = u8::from_str_radix(&hex[0..1], 16).ok().unwrap_or(0);
            let g = u8::from_str_radix(&hex[1..2], 16).ok().unwrap_or(0);
            let b = u8::from_str_radix(&hex[2..3], 16).ok().unwrap_or(0);
            0xFF000000 | ((r as u32 * 17) << 16) | ((g as u32 * 17) << 8) | (b as u32 * 17)
        }
        // #RRGGBB
        6 => {
            let rgb = u32::from_str_radix(hex, 16).ok().unwrap_or(0);
            0xFF000000 | rgb
        }
        // #RRGGBBAA
        8 => {
            let rgba = u32::from_str_radix(hex, 16).ok().unwrap_or(0);
            rgba
        }
        _ => 0x0,
    }
}

pub fn named(str: &str) -> Option<u32> {
    match str {
        "aliceblue" => Some(0xF0F8FF),
        "antiquewhite" => Some(0xFAEBD7),
        "aqua" => Some(0x00FFFF),
        "aquamarine" => Some(0x7FFFD4),
        "azure" => Some(0xF0FFFF),
        "beige" => Some(0xF5F5DC),
        "bisque" => Some(0xFFE4C4),
        "black" => Some(0x000000),
        "blanchedalmond" => Some(0xFFEBCD),
        "blue" => Some(0x0000FF),
        "blueviolet" => Some(0x8A2BE2),
        "brown" => Some(0xA52A2A),
        "burlywood" => Some(0xDEB887),
        "cadetblue" => Some(0x5F9EA0),
        "chartreuse" => Some(0x7FFF00),
        "chocolate" => Some(0xD2691E),
        "coral" => Some(0xFF7F50),
        "cornflowerblue" => Some(0x6495ED),
        "cornsilk" => Some(0xFFF8DC),
        "crimson" => Some(0xDC143C),
        "cyan" => Some(0x00FFFF),
        "darkblue" => Some(0x00008B),
        "darkcyan" => Some(0x008B8B),
        "darkgoldenrod" => Some(0xB8860B),
        "darkgray" => Some(0xA9A9A9),
        "darkgrey" => Some(0xA9A9A9),
        "darkgreen" => Some(0x006400),
        "darkkhaki" => Some(0xBDB76B),
        "darkmagenta" => Some(0x8B008B),
        "darkolivegreen" => Some(0x556B2F),
        "darkorange" => Some(0xFF8C00),
        "darkorchid" => Some(0x9932CC),
        "darkred" => Some(0x8B0000),
        "darksalmon" => Some(0xE9967A),
        "darkseagreen" => Some(0x8FBC8F),
        "darkslateblue" => Some(0x483D8B),
        "darkslategray" => Some(0x2F4F4F),
        "darkslategrey" => Some(0x2F4F4F),
        "darkturquoise" => Some(0x00CED1),
        "darkviolet" => Some(0x9400D3),
        "deeppink" => Some(0xFF1493),
        "deepskyblue" => Some(0x00BFFF),
        "dimgray" => Some(0x696969),
        "dimgrey" => Some(0x696969),
        "dodgerblue" => Some(0x1E90FF),
        "firebrick" => Some(0xB22222),
        "floralwhite" => Some(0xFFFAF0),
        "forestgreen" => Some(0x228B22),
        "fuchsia" => Some(0xFF00FF),
        "gainsboro" => Some(0xDCDCDC),
        "ghostwhite" => Some(0xF8F8FF),
        "gold" => Some(0xFFD700),
        "goldenrod" => Some(0xDAA520),
        "gray" => Some(0x808080),
        "grey" => Some(0x808080),
        "green" => Some(0x008000),
        "greenyellow" => Some(0xADFF2F),
        "honeydew" => Some(0xF0FFF0),
        "hotpink" => Some(0xFF69B4),
        "indianred" => Some(0xCD5C5C),
        "indigo" => Some(0x4B0082),
        "ivory" => Some(0xFFFFF0),
        "khaki" => Some(0xF0E68C),
        "lavender" => Some(0xE6E6FA),
        "lavenderblush" => Some(0xFFF0F5),
        "lawngreen" => Some(0x7CFC00),
        "lemonchiffon" => Some(0xFFFACD),
        "lightblue" => Some(0xADD8E6),
        "lightcoral" => Some(0xF08080),
        "lightcyan" => Some(0xE0FFFF),
        "lightgoldenrodyellow" => Some(0xFAFAD2),
        "lightgray" => Some(0xD3D3D3),
        "lightgrey" => Some(0xD3D3D3),
        "lightgreen" => Some(0x90EE90),
        "lightpink" => Some(0xFFB6C1),
        "lightsalmon" => Some(0xFFA07A),
        "lightseagreen" => Some(0x20B2AA),
        "lightskyblue" => Some(0x87CEFA),
        "lightslategray" => Some(0x778899),
        "lightslategrey" => Some(0x778899),
        "lightsteelblue" => Some(0xB0C4DE),
        "lightyellow" => Some(0xFFFFE0),
        "lime" => Some(0x00FF00),
        "limegreen" => Some(0x32CD32),
        "linen" => Some(0xFAF0E6),
        "magenta" => Some(0xFF00FF),
        "maroon" => Some(0x800000),
        "mediumaquamarine" => Some(0x66CDAA),
        "mediumblue" => Some(0x0000CD),
        "mediumorchid" => Some(0xBA55D3),
        "mediumpurple" => Some(0x9370DB),
        "mediumseagreen" => Some(0x3CB371),
        "mediumslateblue" => Some(0x7B68EE),
        "mediumspringgreen" => Some(0x00FA9A),
        "mediumturquoise" => Some(0x48D1CC),
        "mediumvioletred" => Some(0xC71585),
        "midnightblue" => Some(0x191970),
        "mintcream" => Some(0xF5FFFA),
        "mistyrose" => Some(0xFFE4E1),
        "moccasin" => Some(0xFFE4B5),
        "navajowhite" => Some(0xFFDEAD),
        "navy" => Some(0x000080),
        "oldlace" => Some(0xFDF5E6),
        "olive" => Some(0x808000),
        "olivedrab" => Some(0x6B8E23),
        "orange" => Some(0xFFA500),
        "orangered" => Some(0xFF4500),
        "orchid" => Some(0xDA70D6),
        "palegoldenrod" => Some(0xEEE8AA),
        "palegreen" => Some(0x98FB98),
        "paleturquoise" => Some(0xAFEEEE),
        "palevioletred" => Some(0xDB7093),
        "papayawhip" => Some(0xFFEFD5),
        "peachpuff" => Some(0xFFDAB9),
        "peru" => Some(0xCD853F),
        "pink" => Some(0xFFC0CB),
        "plum" => Some(0xDDA0DD),
        "powderblue" => Some(0xB0E0E6),
        "purple" => Some(0x800080),
        "rebeccapurple" => Some(0x663399),
        "red" => Some(0xFF0000),
        "rosybrown" => Some(0xBC8F8F),
        "royalblue" => Some(0x4169E1),
        "saddlebrown" => Some(0x8B4513),
        "salmon" => Some(0xFA8072),
        "sandybrown" => Some(0xF4A460),
        "seagreen" => Some(0x2E8B57),
        "seashell" => Some(0xFFF5EE),
        "sienna" => Some(0xA0522D),
        "silver" => Some(0xC0C0C0),
        "skyblue" => Some(0x87CEEB),
        "slateblue" => Some(0x6A5ACD),
        "slategray" => Some(0x708090),
        "slategrey" => Some(0x708090),
        "snow" => Some(0xFFFAF0),
        "springgreen" => Some(0x00FF7F),
        "steelblue" => Some(0x4682B4),
        "tan" => Some(0xD2B48C),
        "teal" => Some(0x008080),
        "thistle" => Some(0xD8BFD8),
        "tomato" => Some(0xFF6347),
        "turquoise" => Some(0x40E0D0),
        "violet" => Some(0xEE82EE),
        "wheat" => Some(0xF5DEB3),
        "white" => Some(0xFFFFFF),
        "whitesmoke" => Some(0xF5F5F5),
        "yellow" => Some(0xFFFF00),
        "yellowgreen" => Some(0x9ACD32),
        "transparent" => Some(0x00_000000),
        _ => None,
    }
}