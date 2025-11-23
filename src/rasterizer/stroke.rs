use crate::rasterizer::raster::Line;

pub fn stroke_to_lines(lines: &[Line], stroke_width: f32) -> Vec<Line> {
    if lines.is_empty() {
        return Vec::new();
    }

    let half_width = stroke_width / 2.0;
    let mut stroke_lines = Vec::new();

    let mut left_points = Vec::new();
    let mut right_points = Vec::new();

    for line in lines {
        let dx = line.x1 - line.x0;
        let dy = line.y1 - line.y0;
        let len = (dx * dx + dy * dy).sqrt();

        if len < 1e-6 {
            continue;
        }

        let nx = -dy / len * half_width;
        let ny = dx / len * half_width;

        left_points.push((line.x0 + nx, line.y0 + ny));
        left_points.push((line.x1 + nx, line.y1 + ny));

        right_points.push((line.x0 - nx, line.y0 - ny));
        right_points.push((line.x1 - nx, line.y1 - ny));
    }

    for i in 0..left_points.len() - 1 {
        let (x0, y0) = left_points[i];
        let (x1, y1) = left_points[i + 1];

        stroke_lines.push(Line {
            x0, y0, x1, y1,
            dx: x1 - x0,
            dy: y1 - y0,
            dx_sign: if x1 != x0 { (x1 - x0).signum() as i32 } else { 0 },
            dy_sign: if y1 != y0 { (y1 - y0).signum() as i32 } else { 0 },
            dt_dx: if x1 != x0 { 1.0 / (x1 - x0).abs() } else { f32::MAX },
            dt_dy: if y1 != y0 { 1.0 / (y1 - y0).abs() } else { f32::MAX },
            is_degen: x0 == x1 && y0 == y1,
            abs_dx: (x1 - x0).abs(),
            abs_dy: (y1 - y0).abs(),
            dx_is_zero: x0 == x1,
            dy_is_zero: y0 == y1,
        });
    }

    if let (Some(&(lx, ly)), Some(&(rx, ry))) = (left_points.last(), right_points.last()) {
        stroke_lines.push(Line {
            x0: lx, y0: ly, x1: rx, y1: ry,
            dx: rx - lx,
            dy: ry - ly,
            dx_sign: if rx != lx { (rx - lx).signum() as i32 } else { 0 },
            dy_sign: if ry != ly { (ry - ly).signum() as i32 } else { 0 },
            dt_dx: if rx != lx { 1.0 / (rx - lx).abs() } else { f32::MAX },
            dt_dy: if ry != ly { 1.0 / (ry - ly).abs() } else { f32::MAX },
            is_degen: lx == rx && ly == ry,
            abs_dx: (rx - lx).abs(),
            abs_dy: (ry - ly).abs(),
            dx_is_zero: lx == rx,
            dy_is_zero: ly == ry,
        });
    }

    for i in (1..right_points.len()).rev() {
        let (x0, y0) = right_points[i];
        let (x1, y1) = right_points[i - 1];

        stroke_lines.push(Line {
            x0, y0, x1, y1,
            dx: x1 - x0,
            dy: y1 - y0,
            dx_sign: if x1 != x0 { (x1 - x0).signum() as i32 } else { 0 },
            dy_sign: if y1 != y0 { (y1 - y0).signum() as i32 } else { 0 },
            dt_dx: if x1 != x0 { 1.0 / (x1 - x0).abs() } else { f32::MAX },
            dt_dy: if y1 != y0 { 1.0 / (y1 - y0).abs() } else { f32::MAX },
            is_degen: x0 == x1 && y0 == y1,
            abs_dx: (x1 - x0).abs(),
            abs_dy: (y1 - y0).abs(),
            dx_is_zero: x0 == x1,
            dy_is_zero: y0 == y1,
        });
    }

    if let (Some(&(rx, ry)), Some(&(lx, ly))) = (right_points.first(), left_points.first()) {
        stroke_lines.push(Line {
            x0: rx, y0: ry, x1: lx, y1: ly,
            dx: lx - rx,
            dy: ly - ry,
            dx_sign: if lx != rx { (lx - rx).signum() as i32 } else { 0 },
            dy_sign: if ly != ry { (ly - ry).signum() as i32 } else { 0 },
            dt_dx: if lx != rx { 1.0 / (lx - rx).abs() } else { f32::MAX },
            dt_dy: if ly != ry { 1.0 / (ly - ry).abs() } else { f32::MAX },
            is_degen: rx == lx && ry == ly,
            abs_dx: (lx - rx).abs(),
            abs_dy: (ly - ry).abs(),
            dx_is_zero: rx == lx,
            dy_is_zero: ry == ly,
        });
    }

    stroke_lines
}