use crate::svg::parser::tags::Tag;
use crate::svg::rasterizer::raster::Point;

#[derive(Debug, Clone, Copy)]
pub struct Transform {
    pub a: f32,
    pub b: f32,
    pub c: f32,
    pub d: f32,
    pub e: f32,
    pub f: f32,
}

impl Transform {
    pub fn identity() -> Self {
        Self {
            a: 1.0, c: 0.0, e: 0.0,
            b: 0.0, d: 1.0, f: 0.0,
        }
    }

    pub fn translate(tx: f32, ty: f32) -> Self {
        Self {
            a: 1.0, c: 0.0, e: tx,
            b: 0.0, d: 1.0, f: ty,
        }
    }

    pub fn scale(sx: f32, sy: f32) -> Self {
        Self {
            a: sx,  c: 0.0, e: 0.0,
            b: 0.0, d: sy,  f: 0.0,
        }
    }

    pub fn rotate(angle: f32) -> Self {
        let cos = angle.cos();
        let sin = angle.sin();
        Self {
            a: cos,  c: -sin, e: 0.0,
            b: sin,  d: cos,  f: 0.0,
        }
    }

    pub fn rotate_around(angle: f32, cx: f32, cy: f32) -> Self {
        Self::translate(cx, cy)
            .then(&Self::rotate(angle))
            .then(&Self::translate(-cx, -cy))
    }

    pub fn skew_x(angle: f32) -> Self {
        Self {
            a: 1.0,         c: angle.tan(), e: 0.0,
            b: 0.0,         d: 1.0,         f: 0.0,
        }
    }

    pub fn skew_y(angle: f32) -> Self {
        Self {
            a: 1.0, c: 0.0,         e: 0.0,
            b: angle.tan(), d: 1.0, f: 0.0,
        }
    }

    pub fn matrix(a: f32, b: f32, c: f32, d: f32, e: f32, f: f32) -> Self {
        Self { a, b, c, d, e, f }
    }
    
    pub fn then(&self, other: &Transform) -> Self {
        Self {
            a: self.a * other.a + self.c * other.b,
            b: self.b * other.a + self.d * other.b,
            c: self.a * other.c + self.c * other.d,
            d: self.b * other.c + self.d * other.d,
            e: self.a * other.e + self.c * other.f + self.e,
            f: self.b * other.e + self.d * other.f + self.f,
        }
    }

    pub fn apply(&self, x: f32, y: f32) -> (f32, f32) {
        (
            self.a * x + self.c * y + self.e,
            self.b * x + self.d * y + self.f,
        )
    }

    pub fn apply_point(&self, p: Point) -> Point {
        let (x, y) = self.apply(p.x, p.y);
        Point { x, y }
    }

    pub fn apply_no_translate(&self, x: f32, y: f32) -> (f32, f32) {
        (
            self.a * x + self.c * y,
            self.b * x + self.d * y,
        )
    }
    
    pub fn inverse(&self) -> Option<Self> {
        let det = self.a * self.d - self.b * self.c;
        if det.abs() < 1e-10 {
            return None;
        }
        let inv_det = 1.0 / det;
        Some(Self {
            a: self.d * inv_det,
            b: -self.b * inv_det,
            c: -self.c * inv_det,
            d: self.a * inv_det,
            e: (self.c * self.f - self.d * self.e) * inv_det,
            f: (self.b * self.e - self.a * self.f) * inv_det,
        })
    }

    pub fn get_scale(&self) -> (f32, f32) {
        let sx = (self.a * self.a + self.b * self.b).sqrt();
        let sy = (self.c * self.c + self.d * self.d).sqrt();
        (sx, sy)
    }
    
    pub fn from_str(s: &str) -> Option<Self> {
        let mut result = Transform::identity();
        let mut chars = s.chars().peekable();

        while chars.peek().is_some() {
            while chars.peek().map(|c| c.is_whitespace() || *c == ',').unwrap_or(false) {
                chars.next();
            }

            let mut name = String::new();
            while chars.peek().map(|c| c.is_alphabetic()).unwrap_or(false) {
                name.push(chars.next().unwrap());
            }

            if name.is_empty() {
                break;
            }

            while chars.peek().map(|c| *c != '(').unwrap_or(false) {
                chars.next();
            }
            chars.next();

            let mut args_str = String::new();
            let mut depth = 1;
            while depth > 0 {
                match chars.next() {
                    Some('(') => { depth += 1; args_str.push('('); }
                    Some(')') => { depth -= 1; if depth > 0 { args_str.push(')'); } }
                    Some(c) => args_str.push(c),
                    None => break,
                }
            }

            let args = parse_args(&args_str);

            let t = match name.to_lowercase().as_str() {
                "translate" => {
                    let tx = args.get(0).copied().unwrap_or(0.0);
                    let ty = args.get(1).copied().unwrap_or(0.0);
                    Transform::translate(tx, ty)
                }
                "scale" => {
                    let sx = args.get(0).copied().unwrap_or(1.0);
                    let sy = args.get(1).copied().unwrap_or(sx);
                    Transform::scale(sx, sy)
                }
                "rotate" => {
                    let angle = args.get(0).copied().unwrap_or(0.0).to_radians();
                    if args.len() >= 3 {
                        let cx = args[1];
                        let cy = args[2];
                        Transform::rotate_around(angle, cx, cy)
                    } else {
                        Transform::rotate(angle)
                    }
                }
                "skewx" => {
                    let angle = args.get(0).copied().unwrap_or(0.0).to_radians();
                    Transform::skew_x(angle)
                }
                "skewy" => {
                    let angle = args.get(0).copied().unwrap_or(0.0).to_radians();
                    Transform::skew_y(angle)
                }
                "matrix" => {
                    if args.len() >= 6 {
                        Transform::matrix(args[0], args[1], args[2], args[3], args[4], args[5])
                    } else {
                        Transform::identity()
                    }
                }
                _ => Transform::identity(),
            };

            result = result.then(&t);
        }

        Some(result)
    }
} // This closes the impl Transform block

pub fn parse_transform(tag: &Tag) -> Transform {
    match tag.params.get("transform") {
        Some(s) => Transform::from_str(s).unwrap_or(Transform::identity()),
        None => Transform::identity(),
    }
}

fn parse_args(s: &str) -> Vec<f32> {
    let mut args = Vec::new();
    let mut current = String::new();

    for c in s.chars() {
        if c == ',' || c.is_whitespace() {
            if !current.is_empty() {
                if let Ok(n) = current.parse::<f32>() {
                    args.push(n);
                }
                current.clear();
            }
        } else if c == '-' && !current.is_empty() && !current.ends_with('e') && !current.ends_with('E') {
            if let Ok(n) = current.parse::<f32>() {
                args.push(n);
            }
            current.clear();
            current.push(c);
        } else {
            current.push(c);
        }
    }

    if !current.is_empty() {
        if let Ok(n) = current.parse::<f32>() {
            args.push(n);
        }
    }

    args
}