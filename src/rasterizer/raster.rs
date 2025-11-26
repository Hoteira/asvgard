// Adapted from TiTanF

pub const PI: f32 = 3.14159265;

#[derive(Debug, Clone)]
pub struct Line {
    pub x0: f32,
    pub y0: f32,
    pub x1: f32,
    pub y1: f32,
    pub dx: f32,
    pub dy: f32,
    pub dx_sign: i32,
    pub dy_sign: i32,
    pub dt_dx: f32,
    pub dt_dy: f32,
    pub is_degen: bool,
    pub abs_dx: f32,
    pub abs_dy: f32,
    pub dx_is_zero: bool,
    pub dy_is_zero: bool,
}

impl Line {
    pub fn new(x0: f32, y0: f32, x1: f32, y1: f32, scale: f32) -> Self {
        let dx = x1 - x0;
        let dy = y1 - y0;
        let is_degen = dx == 0.0 && dy == 0.0;

        Self {
            x0: x0 * scale,
            y0: y0 * scale,
            x1: x1 * scale,
            y1: y1 * scale,
            dx: dx * scale,
            dy: dy * scale,
            dx_sign: if dx != 0.0 { dx.signum() as i32 } else { 0 },
            dy_sign: if dy != 0.0 { dy.signum() as i32 } else { 0 },
            dt_dx: if dx != 0.0 { 1.0 / (dx * scale).abs() } else { f32::MAX },
            dt_dy: if dy != 0.0 { 1.0 / (dy * scale).abs() } else { f32::MAX },
            is_degen,
            abs_dx: (dx * scale).abs(),
            abs_dy: (dy * scale).abs(),
            dx_is_zero: dx == 0.0,
            dy_is_zero: dy == 0.0,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

pub struct Segment {
    pub(crate) a_x: f32,
    pub(crate) a_y: f32,
    pub(crate) at: f32,
    pub(crate) c_x: f32,
    pub(crate) c_y: f32,
    pub(crate) ct: f32,
}

impl Segment {
    pub(crate) fn new(a_x: f32, a_y: f32, at: f32, c_x: f32, c_y: f32, ct: f32) -> Self {
        Self { a_x, a_y, at, c_x, c_y, ct }
    }
}

pub struct PathRasterizer {
    pub v_lines: Vec<Line>,
    pub m_lines: Vec<Line>,
    pub lines: Vec<Line>,
    pub bounds: Bounds,
}

#[derive(Debug, Clone, Copy)]
pub struct Bounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

use crate::rasterizer::tags::path::PathCommand;

impl PathRasterizer {
    pub fn new() -> Self {
        Self {
            v_lines: Vec::new(),
            m_lines: Vec::new(),
            lines: Vec::new(),
            bounds: Bounds { x: 0.0, y: 0.0, width: 0.0, height: 0.0 },
        }
    }

    pub fn build_lines_from_path(&mut self, commands: &[PathCommand], scale: f32, tolerance: f32, stroke_width: f32) {
        let max_area = tolerance * tolerance;
        let mut line_segments: Vec<(f32, f32, f32, f32)> = Vec::new();

        let mut current_pos = Point { x: 0.0, y: 0.0 };
        let mut subpath_start = Point { x: 0.0, y: 0.0 };


        for cmd in commands {
            match cmd {
                PathCommand::MoveTo(p) => {
                    current_pos = *p;
                    subpath_start = *p;
                }
                PathCommand::LineTo(p) => {
                    line_segments.push((current_pos.x, current_pos.y, p.x, p.y));
                    current_pos = *p;
                }
                PathCommand::QuadraticBezier(cp, end) => {
                    Self::flatten_quad(
                        current_pos.x, current_pos.y,
                        cp.x, cp.y,
                        end.x, end.y,
                        max_area,
                        &mut line_segments
                    );
                    current_pos = *end;
                }
                PathCommand::CubicBezier(cp1, cp2, end) => {
                    Self::flatten_cubic(
                        current_pos.x, current_pos.y,
                        cp1.x, cp1.y,
                        cp2.x, cp2.y,
                        end.x, end.y,
                        max_area / 9.0,
                        &mut line_segments
                    );
                    current_pos = *end;
                }
                PathCommand::Arc { rx, ry, x_axis_rotation, large_arc_flag, sweep_flag, end } => {
                    Self::flatten_arc(
                        current_pos, *rx, *ry, *x_axis_rotation, *large_arc_flag, *sweep_flag, *end, tolerance,
                        &mut line_segments
                    );
                    current_pos = *end;
                }
                PathCommand::ClosePath => {
                    if current_pos.x != subpath_start.x || current_pos.y != subpath_start.y {
                        line_segments.push((current_pos.x, current_pos.y, subpath_start.x, subpath_start.y));
                    }
                    current_pos = subpath_start;
                }
            }
        }


        if line_segments.is_empty() {
            self.bounds = Bounds { x: 0.0, y: 0.0, width: 0.0, height: 0.0 };
            return;
        }

        let mut x_min = f32::MAX;
        let mut x_max = f32::MIN;
        let mut y_min = f32::MAX;
        let mut y_max = f32::MIN;

        for (x0, y0, x1, y1) in &line_segments {
            x_min = x_min.min(*x0).min(*x1);
            x_max = x_max.max(*x0).max(*x1);
            y_min = y_min.min(*y0).min(*y1);
            y_max = y_max.max(*y0).max(*y1);
        }

        let half_width = stroke_width / 2.0;
        x_min -= half_width;
        x_max += half_width;
        y_min -= half_width;
        y_max += half_width;

        self.bounds = Bounds {
            x: x_min * scale,
            y: y_min * scale,
            width: (x_max - x_min) * scale,
            height: (y_max - y_min) * scale,
        };


        for (x0, y0, x1, y1) in line_segments {
            self.insert_line(x0, y0, x1, y1, scale);
        }
    }

    fn flatten_quad(
        p0_x: f32, p0_y: f32,
        p1_x: f32, p1_y: f32,
        p2_x: f32, p2_y: f32,
        max_area: f32,
        output: &mut Vec<(f32, f32, f32, f32)>
    ) {
        let mut stack = vec![Segment::new(p0_x, p0_y, 0.0, p2_x, p2_y, 1.0)];

        while let Some(seg) = stack.pop() {
            let bt = (seg.at + seg.ct) * 0.5;
            let tm = 1.0 - bt;
            let a = tm * tm;
            let b = 2.0 * tm * bt;
            let c = bt * bt;
            let b_x = a * p0_x + b * p1_x + c * p2_x;
            let b_y = a * p0_y + b * p1_y + c * p2_y;

            let area = (b_x - seg.a_x) * (seg.c_y - seg.a_y) - (seg.c_x - seg.a_x) * (b_y - seg.a_y);

            if area.abs() > max_area {
                stack.push(Segment::new(b_x, b_y, bt, seg.c_x, seg.c_y, seg.ct));

                stack.push(Segment::new(seg.a_x, seg.a_y, seg.at, b_x, b_y, bt));
            } else {
                output.push((seg.a_x, seg.a_y, seg.c_x, seg.c_y));
            }
        }
    }

    fn flatten_cubic(
        p0_x: f32, p0_y: f32,
        p1_x: f32, p1_y: f32,
        p2_x: f32, p2_y: f32,
        p3_x: f32, p3_y: f32,
        tolerance_sq: f32,
        output: &mut Vec<(f32, f32, f32, f32)>
    ) {
        let mut stack = vec![(p0_x, p0_y, p1_x, p1_y, p2_x, p2_y, p3_x, p3_y)];

        while let Some((x0, y0, x1, y1, x2, y2, x3, y3)) = stack.pop() {
            let dx = x3 - x0;
            let dy = y3 - y0;
            let length_sq = dx * dx + dy * dy;

            if length_sq < 1e-9 {
                continue;
            }

            let d1 = (x1 - x0) * dy - (y1 - y0) * dx;
            let d2 = (x2 - x0) * dy - (y2 - y0) * dx;
            let d1_sq = d1 * d1;
            let d2_sq = d2 * d2;

            if d1_sq.max(d2_sq) < tolerance_sq * length_sq {
                output.push((x0, y0, x3, y3));
                continue;
            }

            let x01 = (x0 + x1) * 0.5;
            let y01 = (y0 + y1) * 0.5;
            let x12 = (x1 + x2) * 0.5;
            let y12 = (y1 + y2) * 0.5;
            let x23 = (x2 + x3) * 0.5;
            let y23 = (y2 + y3) * 0.5;

            let x012 = (x01 + x12) * 0.5;
            let y012 = (y01 + y12) * 0.5;
            let x123 = (x12 + x23) * 0.5;
            let y123 = (y12 + y23) * 0.5;

            let x0123 = (x012 + x123) * 0.5;
            let y0123 = (y012 + y123) * 0.5;

            stack.push((x0123, y0123, x123, y123, x23, y23, x3, y3));
            stack.push((x0, y0, x01, y01, x012, y012, x0123, y0123));
        }
    }

    fn flatten_arc(
        start: Point,
        rx: f32, ry: f32,
        x_axis_rotation: f32,
        large_arc: bool,
        sweep: bool,
        end: Point,
        tolerance: f32,
        output: &mut Vec<(f32, f32, f32, f32)>
    ) {
        if rx == 0.0 || ry == 0.0 {
            output.push((start.x, start.y, end.x, end.y));
            return;
        }

        let phi = x_axis_rotation * PI / 180.0;
        let cos_phi = phi.cos();
        let sin_phi = phi.sin();

        let dx = (start.x - end.x) / 2.0;
        let dy = (start.y - end.y) / 2.0;

        let x1 = cos_phi * dx + sin_phi * dy;
        let y1 = -sin_phi * dx + cos_phi * dy;

        let mut rx = rx.abs();
        let mut ry = ry.abs();

        let lambda = (x1 * x1) / (rx * rx) + (y1 * y1) / (ry * ry);
        if lambda > 1.0 {
            rx *= lambda.sqrt();
            ry *= lambda.sqrt();
        }

        let sq = ((rx * rx * ry * ry - rx * rx * y1 * y1 - ry * ry * x1 * x1) /
            (rx * rx * y1 * y1 + ry * ry * x1 * x1)).max(0.0).sqrt();

        let sign = if large_arc == sweep { -1.0 } else { 1.0 };
        let cx1 = sign * sq * rx * y1 / ry;
        let cy1 = -sign * sq * ry * x1 / rx;

        let cx = cos_phi * cx1 - sin_phi * cy1 + (start.x + end.x) / 2.0;
        let cy = sin_phi * cx1 + cos_phi * cy1 + (start.y + end.y) / 2.0;


        let start_angle = ((y1 - cy1) / ry).atan2((x1 - cx1) / rx);
        let end_angle = ((-y1 - cy1) / ry).atan2((-x1 - cx1) / rx);

        let mut delta_angle = end_angle - start_angle;
        if sweep && delta_angle < 0.0 {
            delta_angle += 2.0 * PI;
        } else if !sweep && delta_angle > 0.0 {
            delta_angle -= 2.0 * PI;
        }


        let num_segments = ((delta_angle.abs() * rx.max(ry)) / tolerance).ceil() as usize;
        let num_segments = num_segments.max(4).min(100);

        let mut prev_x = start.x;
        let mut prev_y = start.y;

        for i in 1..=num_segments {
            let angle = start_angle + delta_angle * (i as f32 / num_segments as f32);
            let cos_angle = angle.cos();
            let sin_angle = angle.sin();

            let x = cx + cos_phi * rx * cos_angle - sin_phi * ry * sin_angle;
            let y = cy + sin_phi * rx * cos_angle + cos_phi * ry * sin_angle;

            output.push((prev_x, prev_y, x, y));
            prev_x = x;
            prev_y = y;
        }
    }

    pub(crate) fn insert_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, scale: f32) {
        let line = Line::new(x0, y0, x1, y1, scale);
        let dy = line.dy;
        let dx = line.dx;

        self.lines.push(line.clone());

        if dy != 0.0 {
            if dx == 0.0 {
                self.v_lines.push(line)
            } else {
                self.m_lines.push(line);
            }
        }
    }
}
