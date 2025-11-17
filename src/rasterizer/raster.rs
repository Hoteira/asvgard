use std::f32;
use crate::rasterizer::tags::path::PathCommand;

// Adapted from my previous rasterizer TiTanFont

#[derive(Debug, Clone, Copy)]
pub struct Point {
    pub x: f32,
    pub y: f32,
}

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

struct Segment {
    a_x: f32,
    a_y: f32,
    at: f32,
    c_x: f32,
    c_y: f32,
    ct: f32,
}

impl Segment {
    fn new(a_x: f32, a_y: f32, at: f32, c_x: f32, c_y: f32, ct: f32) -> Self {
        Self { a_x, a_y, at, c_x, c_y, ct }
    }
}

pub struct PathRasterizer {
    pub v_lines: Vec<Line>,
    pub m_lines: Vec<Line>,
    pub bounds: Bounds,
}

#[derive(Debug, Clone, Copy)]
pub struct Bounds {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl PathRasterizer {
    pub fn new() -> Self {
        Self {
            v_lines: Vec::new(),
            m_lines: Vec::new(),
            bounds: Bounds { x: 0.0, y: 0.0, width: 0.0, height: 0.0 },
        }
    }

    pub fn build_lines_from_path(&mut self, commands: &[PathCommand], scale: f32, tolerance: f32) {
        let max_area = tolerance * tolerance;
        let mut line_segments: Vec<(f32, f32, f32, f32)> = Vec::new();

        let mut x_min = f32::MAX;
        let mut x_max = f32::MIN;
        let mut y_min = f32::MAX;
        let mut y_max = f32::MIN;

        let mut current_pos = Point { x: 0.0, y: 0.0 };
        let mut subpath_start = Point { x: 0.0, y: 0.0 };

        for cmd in commands {
            match cmd {
                PathCommand::MoveTo(p) => {
                    current_pos = *p;
                    subpath_start = *p;

                    x_min = x_min.min(p.x);
                    x_max = x_max.max(p.x);
                    y_min = y_min.min(p.y);
                    y_max = y_max.max(p.y);
                }
                PathCommand::LineTo(p) => {
                    line_segments.push((current_pos.x, current_pos.y, p.x, p.y));

                    x_min = x_min.min(p.x);
                    x_max = x_max.max(p.x);
                    y_min = y_min.min(p.y);
                    y_max = y_max.max(p.y);

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

                    x_min = x_min.min(cp.x).min(end.x);
                    x_max = x_max.max(cp.x).max(end.x);
                    y_min = y_min.min(cp.y).min(end.y);
                    y_max = y_max.max(cp.y).max(end.y);

                    current_pos = *end;
                }
                PathCommand::CubicBezier(cp1, cp2, end) => {
                    Self::flatten_cubic(
                        current_pos.x, current_pos.y,
                        cp1.x, cp1.y,
                        cp2.x, cp2.y,
                        end.x, end.y,
                        max_area,
                        &mut line_segments
                    );

                    x_min = x_min.min(cp1.x).min(cp2.x).min(end.x);
                    x_max = x_max.max(cp1.x).max(cp2.x).max(end.x);
                    y_min = y_min.min(cp1.y).min(cp2.y).min(end.y);
                    y_max = y_max.max(cp1.y).max(cp2.y).max(end.y);

                    current_pos = *end;
                }
                PathCommand::Arc { rx, ry, x_axis_rotation, large_arc_flag, sweep_flag, end } => {
                    Self::flatten_arc(
                        current_pos,
                        *rx, *ry,
                        *x_axis_rotation,
                        *large_arc_flag,
                        *sweep_flag,
                        *end,
                        tolerance,
                        &mut line_segments
                    );

                    x_min = x_min.min(end.x);
                    x_max = x_max.max(end.x);
                    y_min = y_min.min(end.y);
                    y_max = y_max.max(end.y);

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

        for (x0, y0, x1, y1) in line_segments {
            self.insert_line(x0, y0, x1, y1, scale);
        }

        for line in self.v_lines.iter_mut().chain(self.m_lines.iter_mut()) {
            line.x0 -= x_min;
            line.y0 -= y_min;
            line.x1 -= x_min;
            line.y1 -= y_min;
        }

        let width = x_max - x_min;
        let height = y_max - y_min;

        for line in self.v_lines.iter_mut().chain(self.m_lines.iter_mut()) {

            line.dx = line.x1 - line.x0;
            line.dy = line.y1 - line.y0;
            line.dx_is_zero = line.dx.abs() < 1e-6;
            line.dy_is_zero = line.dy.abs() < 1e-6;
            line.dx_sign = line.dx.signum() as i32;
            line.dy_sign = line.dy.signum() as i32;
            line.dt_dx = if !line.dx_is_zero { 1.0 / line.dx.abs() } else { f32::MAX };
            line.dt_dy = if !line.dy_is_zero { 1.0 / line.dy.abs() } else { f32::MAX };
            line.is_degen = line.dx_is_zero && line.dy_is_zero;
            line.abs_dx = line.dx.abs();
            line.abs_dy = line.dy.abs();
        }

        self.bounds = Bounds {
            x: x_min,
            y: y_min,
            width,
            height,
        };
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
                stack.push(Segment::new(seg.a_x, seg.a_y, seg.at, b_x, b_y, bt));
                stack.push(Segment::new(b_x, b_y, bt, seg.c_x, seg.c_y, seg.ct));
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
        max_area: f32,
        output: &mut Vec<(f32, f32, f32, f32)>
    ) {

        let mut stack = vec![(0.0, 1.0, p0_x, p0_y, p3_x, p3_y)];

        while let Some((t0, t1, start_x, start_y, end_x, end_y)) = stack.pop() {
            let t_mid = (t0 + t1) * 0.5;


            let tm = 1.0 - t_mid;
            let tm2 = tm * tm;
            let tm3 = tm2 * tm;
            let t2 = t_mid * t_mid;
            let t3 = t2 * t_mid;

            let mid_x = tm3 * p0_x + 3.0 * tm2 * t_mid * p1_x + 3.0 * tm * t2 * p2_x + t3 * p3_x;
            let mid_y = tm3 * p0_y + 3.0 * tm2 * t_mid * p1_y + 3.0 * tm * t2 * p2_y + t3 * p3_y;


            let area = (mid_x - start_x) * (end_y - start_y) - (end_x - start_x) * (mid_y - start_y);

            if area.abs() > max_area {
                stack.push((t_mid, t1, mid_x, mid_y, end_x, end_y));
                stack.push((t0, t_mid, start_x, start_y, mid_x, mid_y));
            } else {
                output.push((start_x, start_y, end_x, end_y));
            }
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

        let phi = x_axis_rotation * f32::consts::PI / 180.0;
        let cos_phi = phi.cos();
        let sin_phi = phi.sin();

        // Convert to center parameterization
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
            delta_angle += 2.0 * f32::consts::PI;
        } else if !sweep && delta_angle > 0.0 {
            delta_angle -= 2.0 * f32::consts::PI;
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

    fn insert_line(&mut self, x0: f32, y0: f32, x1: f32, y1: f32, scale: f32) {
        if y0 == y1 {
            return;
        }

        let dx = x1 - x0;
        let dy = y1 - y0;
        let is_degen = dx == 0.0 && dy == 0.0;

        let line = Line {
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
        };

        if x0 == x1 {
            self.v_lines.push(line);
        } else {
            self.m_lines.push(line);
        }
    }
}