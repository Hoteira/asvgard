use crate::rasterizer::canva::Canvas;
use crate::rasterizer::dda::Rasterizer;
use crate::rasterizer::raster::{Bounds, Line, PathRasterizer};
use crate::rasterizer::tags::path::generate_color_map;
use crate::utils::color::Paint;

fn create_stroke_outline(segments: &[Line], stroke_width: f32) -> Vec<Line> {
    if segments.is_empty() {
        return Vec::new();
    }

    let half_width = stroke_width / 2.0;
    let mut outline = Vec::new();
    let mut rectangles = Vec::new();

    for segment in segments {
        let dx = segment.x1 - segment.x0;
        let dy = segment.y1 - segment.y0;
        let len = (dx * dx + dy * dy).sqrt();

        let (nx_scaled, ny_scaled) = if len > 1e-9 {
            (-dy / len * half_width, dx / len * half_width)
        } else {
            (0.0, 0.0)
        };

        let p0_left = (segment.x0 + nx_scaled, segment.y0 + ny_scaled);
        let p1_left = (segment.x1 + nx_scaled, segment.y1 + ny_scaled);
        let p1_right = (segment.x1 - nx_scaled, segment.y1 - ny_scaled);
        let p0_right = (segment.x0 - nx_scaled, segment.y0 - ny_scaled);

        rectangles.push((p0_left, p1_left, p1_right, p0_right));
    }

    if rectangles.is_empty() {
        return outline;
    }

    let is_closed = if segments.len() >= 1 {
        let first_segment = &segments[0];
        let last_segment = segments.last().unwrap();
        (first_segment.x0 - last_segment.x1).abs() < 1e-3
            && (first_segment.y0 - last_segment.y1).abs() < 1e-3
    } else {
        false
    };

    if is_closed {
        for i in 0..rectangles.len() {
            let (_, _, p1_right, p0_right) = rectangles[i];
            outline.push(make_line(p0_right.0, p0_right.1, p1_right.0, p1_right.1));
            let next_i = (i + 1) % rectangles.len();
            let (_, _, _, next_p0_right) = rectangles[next_i];
            outline.push(make_line(p1_right.0, p1_right.1, next_p0_right.0, next_p0_right.1));
        }

        for i in (0..rectangles.len()).rev() {
            let (p0_left, p1_left, _, _) = rectangles[i];
            outline.push(make_line(p1_left.0, p1_left.1, p0_left.0, p0_left.1));
            let prev_i = if i == 0 { rectangles.len() - 1 } else { i - 1 };
            let (_, prev_p1_left, _, _) = rectangles[prev_i];
            outline.push(make_line(p0_left.0, p0_left.1, prev_p1_left.0, prev_p1_left.1));
        }
    } else {
        let first_rect = rectangles[0];
        let last_rect = rectangles[rectangles.len() - 1];

        let first_line = &segments[0];
        let (cap_tx, cap_ty) = if !first_line.is_degen {
            (-(first_line.x1 - first_line.x0), -(first_line.y1 - first_line.y0))
        } else { (0.0, 0.0) };
        let len = (cap_tx * cap_tx + cap_ty * cap_ty).sqrt();
        let (cap_tx_scaled, cap_ty_scaled) = if len > 1e-9 {
            (cap_tx / len * half_width, cap_ty / len * half_width)
        } else { (0.0, 0.0) };

        let start_p0_left_cap = (first_rect.0.0 + cap_tx_scaled, first_rect.0.1 + cap_ty_scaled);
        let start_p0_right_cap = (first_rect.3.0 + cap_tx_scaled, first_rect.3.1 + cap_ty_scaled);

        outline.push(make_line(first_rect.0.0, first_rect.0.1, start_p0_left_cap.0, start_p0_left_cap.1));
        outline.push(make_line(start_p0_left_cap.0, start_p0_left_cap.1, start_p0_right_cap.0, start_p0_right_cap.1));
        outline.push(make_line(start_p0_right_cap.0, start_p0_right_cap.1, first_rect.3.0, first_rect.3.1));

        for i in 0..rectangles.len() {
            let (_, _, p1_right, p0_right) = rectangles[i];
            outline.push(make_line(p0_right.0, p0_right.1, p1_right.0, p1_right.1));
            if i + 1 < rectangles.len() {
                let (_, _, _, next_p0_right) = rectangles[i + 1];
                outline.push(make_line(p1_right.0, p1_right.1, next_p0_right.0, next_p0_right.1));
            }
        }

        let last_line = segments.last().unwrap();
        let (cap_tx, cap_ty) = if !last_line.is_degen {
            (last_line.x1 - last_line.x0, last_line.y1 - last_line.y0)
        } else { (0.0, 0.0) };
        let len = (cap_tx * cap_tx + cap_ty * cap_ty).sqrt();
        let (cap_tx_scaled, cap_ty_scaled) = if len > 1e-9 {
            (cap_tx / len * half_width, cap_ty / len * half_width)
        } else { (0.0, 0.0) };

        let end_p1_left_cap = (last_rect.1.0 + cap_tx_scaled, last_rect.1.1 + cap_ty_scaled);
        let end_p1_right_cap = (last_rect.2.0 + cap_tx_scaled, last_rect.2.1 + cap_ty_scaled);

        outline.push(make_line(last_rect.2.0, last_rect.2.1, end_p1_right_cap.0, end_p1_right_cap.1));
        outline.push(make_line(end_p1_right_cap.0, end_p1_right_cap.1, end_p1_left_cap.0, end_p1_left_cap.1));
        outline.push(make_line(end_p1_left_cap.0, end_p1_left_cap.1, last_rect.1.0, last_rect.1.1));

        for i in (0..rectangles.len()).rev() {
            let (p0_left, p1_left, _, _) = rectangles[i];
            outline.push(make_line(p1_left.0, p1_left.1, p0_left.0, p0_left.1));
            if i > 0 {
                let (_, prev_p1_left, _, _) = rectangles[i - 1];
                outline.push(make_line(p0_left.0, p0_left.1, prev_p1_left.0, prev_p1_left.1));
            }
        }
    }

    outline
}

fn make_line(x0: f32, y0: f32, x1: f32, y1: f32) -> Line {
    Line::new(x0, y0, x1, y1, 1.0)
}

fn translate_lines(lines: &[Line], dx: f32, dy: f32) -> Vec<Line> {
    lines.iter().map(|line| {
        let mut new_line = line.clone();
        new_line.x0 -= dx;
        new_line.y0 -= dy;
        new_line.x1 -= dx;
        new_line.y1 -= dy;
        new_line
    }).collect()
}

pub fn draw_stroke(map: &mut Canvas, rasterizer: &PathRasterizer, stroke: Paint, stroke_width: f32) {
    let stroke_outline_segments = create_stroke_outline(&rasterizer.lines, stroke_width);

    if stroke_outline_segments.is_empty() {
        return;
    }

    let mut stroke_path_rasterizer = PathRasterizer::new();
    let mut x_min = f32::MAX;
    let mut x_max = f32::MIN;
    let mut y_min = f32::MAX;
    let mut y_max = f32::MIN;

    for line in &stroke_outline_segments {
        x_min = x_min.min(line.x0).min(line.x1);
        x_max = x_max.max(line.x0).max(line.x1);
        y_min = y_min.min(line.y0).min(line.y1);
        y_max = y_max.max(line.y0).max(line.y1);
    }
    
    if x_min >= x_max || y_min >= y_max {
        return; 
    }

    for line in stroke_outline_segments {
        stroke_path_rasterizer.insert_line(line.x0, line.y0, line.x1, line.y1, 1.0);
    }

    let bounds = Bounds {
        x: x_min,
        y: y_min,
        width: x_max - x_min,
        height: y_max - y_min,
    };

    let r_w = bounds.width.ceil() as usize + 1;
    let r_h = bounds.height.ceil() as usize;
    let draw_x = bounds.x.round() as usize;
    let draw_y = bounds.y.round() as usize;

    if r_w == 0 || r_h == 0 {
        return;
    }
    
    let local_v = translate_lines(&stroke_path_rasterizer.v_lines, bounds.x, bounds.y);
    let local_m = translate_lines(&stroke_path_rasterizer.m_lines, bounds.x, bounds.y);

    let mut stroke_renderer = Rasterizer::new(r_w, r_h);
    let stroke_bitmap = stroke_renderer
        .draw(&local_v, &local_m)
        .to_bitmap();

    let stroke_color_map = generate_color_map(
        &stroke_bitmap,
        &stroke,
        r_w,
        r_h,
        bounds.x,
        bounds.y,
    );

    map.add_buffer(&stroke_color_map, draw_x, draw_y, r_w, r_h);
}
