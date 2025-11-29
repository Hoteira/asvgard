use crate::svg::parser::tags::Tag;
use crate::svg::rasterizer::canva::Canvas;
use crate::svg::rasterizer::dda::Rasterizer;
use crate::svg::rasterizer::raster::{Line, PathRasterizer, Point};
use crate::svg::utils::color::{Paint, get_fill, get_stroke};
use crate::svg::utils::transform::Transform;
use crate::utils::compat::HashMap;
use crate::svg::rasterizer::stroke::draw_stroke;
use crate::svg::utils::effects::get_stroke_width;
use crate::utils::compat::FloatExt;
use crate::utils::compat::{Vec, String};

#[derive(Debug, Clone)]
pub enum PathCommand {
    MoveTo(Point),
    LineTo(Point),
    CubicBezier(Point, Point, Point),
    QuadraticBezier(Point, Point),
    Arc {
        rx: f32,
        ry: f32,
        x_axis_rotation: f32,
        large_arc_flag: bool,
        sweep_flag: bool,
        end: Point,
    },
    ClosePath,
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


pub fn draw_path(
    tag: &mut Tag,
    defs: &HashMap<String, Tag>,
    map: &mut Canvas,
    transform: &Transform
) {
    let d = match tag.params.get("d") {
        Some(d) => d,
        None => return,
    };

    let d_path = parse_path_data(d);
    let transformed_path = apply_transform_to_path(&d_path, transform);

    let mut fill = get_fill(tag).resolve(defs);
    let mut stroke = get_stroke(tag).resolve(defs);
    let stroke_width = get_stroke_width(tag);

    let (sx, sy) = transform.get_scale();
    let scale = sx.min(sy);

    fill.scale(scale);
    stroke.scale(scale);


    if !fill.is_none() {
    
        let mut fill_commands = Vec::new();
        let mut subpath_open = false;
        for cmd in &transformed_path {
            match cmd {
                PathCommand::MoveTo(_) => {
                    if subpath_open {
                        fill_commands.push(PathCommand::ClosePath);
                    }
                    fill_commands.push(cmd.clone());
                    subpath_open = true;
                }
                PathCommand::ClosePath => {
                    fill_commands.push(cmd.clone());
                    subpath_open = false;
                }
                _ => {
                    fill_commands.push(cmd.clone());
                }
            }
        }
        if subpath_open {
            fill_commands.push(PathCommand::ClosePath);
        }

        let mut fill_rasterizer = PathRasterizer::new();

        fill_rasterizer.build_lines_from_path(&fill_commands, 1.0, 1.0, 0.0);

        let bounds = fill_rasterizer.bounds;

        let r_w = bounds.width.ceil() as usize + 1;
        let r_h = bounds.height.ceil() as usize;

        if r_w > 0 && r_h > 0 {
            let draw_x = bounds.x.round() as usize;
            let draw_y = bounds.y.round() as usize;
            let local_v = translate_lines(&fill_rasterizer.v_lines, bounds.x, bounds.y);
            let local_m = translate_lines(&fill_rasterizer.m_lines, bounds.x, bounds.y);
            let renderer = Rasterizer::new(r_w, r_h);
            let bitmap = renderer.draw(&local_v, &local_m).to_bitmap();
            let color_map = generate_color_map(&bitmap, &fill, r_w, r_h, bounds.x, bounds.y, bounds.x, bounds.y, bounds.width, bounds.height);
            map.add_buffer(&color_map, draw_x as isize, draw_y as isize, r_w, r_h);
        } else {
            #[cfg(feature = "std")]
            std::println!("Skipping draw: invalid bounds dimensions {}x{}", r_w, r_h);
        }
    }


    if !stroke.is_none() && stroke_width > 0.0 {
        let mut subpath_start_index = 0;
        for (i, cmd) in transformed_path.iter().enumerate() {
            if i > 0 && matches!(cmd, PathCommand::MoveTo(_)) {
                let subpath = &transformed_path[subpath_start_index..i];
                if !subpath.is_empty() {
                    let mut stroke_rasterizer = PathRasterizer::new();
                    stroke_rasterizer.build_lines_from_path(subpath, 1.0, 1.0, stroke_width);
                    draw_stroke(map, &stroke_rasterizer, stroke.clone(), stroke_width, 0.0, 0.0, 0.0, 0.0);
                }
                subpath_start_index = i;
            }
        }
        let subpath = &transformed_path[subpath_start_index..];
        if !subpath.is_empty() {
            let mut stroke_rasterizer = PathRasterizer::new();
            stroke_rasterizer.build_lines_from_path(subpath, 1.0, 1.0, stroke_width);
            draw_stroke(map, &stroke_rasterizer, stroke, stroke_width, 0.0, 0.0, 0.0, 0.0);
        }
    }
}

pub fn parse_path_data(d: &str) -> Vec<PathCommand> {
    let mut d_path: Vec<PathCommand> = Vec::new();
    let mut current_command = ' ';
    let mut current_number = String::new();
    let mut args: Vec<f32> = Vec::new();
    let mut current_pos = Point { x: 0.0, y: 0.0 };
    let mut subpath_start = Point { x: 0.0, y: 0.0 };

    for c in d.chars() {
        if c.is_whitespace() || c == ',' {
            if !current_number.is_empty() {
                args.push(current_number.parse::<f32>().unwrap_or(0.0));
                current_number.clear();
            }
            continue;
        }

        if c.is_alphabetic() {
            if !current_number.is_empty() {
                args.push(current_number.parse::<f32>().unwrap_or(0.0));
                current_number.clear();
            }

            if current_command != ' ' {
                if !args.is_empty() || current_command.to_ascii_uppercase() == 'Z' {
                    process_command(current_command, &mut args, &mut d_path, &mut current_pos, &mut subpath_start);
                }
            }

            current_command = c;
            continue;
        }

        if (c == '-' || c == '+') && !current_number.is_empty() {
            let last_char = current_number.chars().last().unwrap();
            if last_char == 'e' || last_char == 'E' {
                current_number.push(c);
            } else {
                args.push(current_number.parse::<f32>().unwrap_or(0.0));
                current_number.clear();
                current_number.push(c);
            }
        } else {
            current_number.push(c);
        }
    }

    if !current_number.is_empty() {
        args.push(current_number.parse::<f32>().unwrap_or(0.0));
    }

    if current_command != ' ' {
        process_command(current_command, &mut args, &mut d_path, &mut current_pos, &mut subpath_start);
    }

    d_path
}

fn process_command(
    command: char,
    args: &mut Vec<f32>,
    d_path: &mut Vec<PathCommand>,
    current_pos: &mut Point,
    subpath_start: &mut Point
) {
    let is_relative = command.is_lowercase();
    let cmd = command.to_ascii_uppercase();

    match cmd {
        'M' => {
            let mut i = 0;
            while i + 1 < args.len() {
                let (x, y) = if is_relative {
                    (current_pos.x + args[i], current_pos.y + args[i + 1])
                } else {
                    (args[i], args[i + 1])
                };

                if i == 0 {
                    d_path.push(PathCommand::MoveTo(Point { x, y }));
                    *subpath_start = Point { x, y };
                } else {
                    d_path.push(PathCommand::LineTo(Point { x, y }));
                }

                *current_pos = Point { x, y };
                i += 2;
            }
        }
        'L' => {
            let mut i = 0;
            while i + 1 < args.len() {
                let (x, y) = if is_relative {
                    (current_pos.x + args[i], current_pos.y + args[i + 1])
                } else {
                    (args[i], args[i + 1])
                };

                d_path.push(PathCommand::LineTo(Point { x, y }));
                *current_pos = Point { x, y };
                i += 2;
            }
        }
        'H' => {
            for &val in args.iter() {
                let x = if is_relative { current_pos.x + val } else { val };
                d_path.push(PathCommand::LineTo(Point { x, y: current_pos.y }));
                current_pos.x = x;
            }
        }
        'V' => {
            for &val in args.iter() {
                let y = if is_relative { current_pos.y + val } else { val };
                d_path.push(PathCommand::LineTo(Point { x: current_pos.x, y }));
                current_pos.y = y;
            }
        }
        'C' => {
            let mut i = 0;
            while i + 5 < args.len() {
                let (x1, y1, x2, y2, x, y) = if is_relative {
                    (
                        current_pos.x + args[i], current_pos.y + args[i + 1],
                        current_pos.x + args[i + 2], current_pos.y + args[i + 3],
                        current_pos.x + args[i + 4], current_pos.y + args[i + 5]
                    )
                } else {
                    (args[i], args[i + 1], args[i + 2], args[i + 3], args[i + 4], args[i + 5])
                };

                d_path.push(PathCommand::CubicBezier(
                    Point { x: x1, y: y1 },
                    Point { x: x2, y: y2 },
                    Point { x, y }
                ));
                *current_pos = Point { x, y };
                i += 6;
            }
        }
        'Q' => {
            let mut i = 0;
            while i + 3 < args.len() {
                let (x1, y1, x, y) = if is_relative {
                    (
                        current_pos.x + args[i], current_pos.y + args[i + 1],
                        current_pos.x + args[i + 2], current_pos.y + args[i + 3]
                    )
                } else {
                    (args[i], args[i + 1], args[i + 2], args[i + 3])
                };

                d_path.push(PathCommand::QuadraticBezier(
                    Point { x: x1, y: y1 },
                    Point { x, y }
                ));
                *current_pos = Point { x, y };
                i += 4;
            }
        }
        'A' => {
            let mut i = 0;
            while i + 6 < args.len() {
                let (x, y) = if is_relative {
                    (current_pos.x + args[i + 5], current_pos.y + args[i + 6])
                } else {
                    (args[i + 5], args[i + 6])
                };

                d_path.push(PathCommand::Arc {
                    rx: args[i],
                    ry: args[i + 1],
                    x_axis_rotation: args[i + 2],
                    large_arc_flag: args[i + 3] != 0.0,
                    sweep_flag: args[i + 4] != 0.0,
                    end: Point { x, y },
                });
                *current_pos = Point { x, y };
                i += 7;
            }
        }
        'T' => {
            let mut i = 0;
            while i + 1 < args.len() {
                let prev_control = if let Some(PathCommand::QuadraticBezier(cp, _)) = d_path.last() {
                    Point {
                        x: 2.0 * current_pos.x - cp.x,
                        y: 2.0 * current_pos.y - cp.y,
                    }
                } else {
                    *current_pos
                };

                let (x, y) = if is_relative {
                    (current_pos.x + args[i], current_pos.y + args[i + 1])
                } else {
                    (args[i], args[i + 1])
                };

                d_path.push(PathCommand::QuadraticBezier(
                    prev_control,
                    Point { x, y }
                ));
                *current_pos = Point { x, y };
                i += 2;
            }
        },
        'Z' => {
            d_path.push(PathCommand::ClosePath);
            *current_pos = *subpath_start;
        }
        _ => {}
    }

    args.clear();
}

fn arc_to_beziers(start: Point, rx: f32, ry: f32, x_axis_rotation: f32, large_arc: bool, sweep: bool, end: Point) -> Vec<PathCommand> {
    let pi = core::f32::consts::PI;
    let cos_phi = (x_axis_rotation * pi / 180.0).cos();
    let sin_phi = (x_axis_rotation * pi / 180.0).sin();

    let mut rx = rx.abs();
    let mut ry = ry.abs();

    let dx2 = (start.x - end.x) / 2.0;
    let dy2 = (start.y - end.y) / 2.0;

    let x1p = cos_phi * dx2 + sin_phi * dy2;
    let y1p = -sin_phi * dx2 + cos_phi * dy2;

    let lambda_check = (x1p * x1p) / (rx * rx) + (y1p * y1p) / (ry * ry);
    if lambda_check > 1.0 {
        rx *= lambda_check.sqrt();
        ry *= lambda_check.sqrt();
    }

    let sign = if large_arc == sweep { -1.0 } else { 1.0 };
    let num = rx * rx * ry * ry - rx * rx * y1p * y1p - ry * ry * x1p * x1p;
    let den = rx * rx * y1p * y1p + ry * ry * x1p * x1p;
    let mut sq = (num / den).max(0.0).sqrt();
    if den < 1e-9 { sq = 0.0; }
    let cxp = sign * sq * (rx * y1p / ry);
    let cyp = sign * sq * -(ry * x1p / rx);

    let cx = cos_phi * cxp - sin_phi * cyp + (start.x + end.x) / 2.0;
    let cy = sin_phi * cxp + cos_phi * cyp + (start.y + end.y) / 2.0;

    let start_vec_x = (x1p - cxp) / rx;
    let start_vec_y = (y1p - cyp) / ry;
    let end_vec_x = (-x1p - cxp) / rx;
    let end_vec_y = (-y1p - cyp) / ry;
    
    let start_angle = start_vec_y.atan2(start_vec_x);
    let dot = (start_vec_x * end_vec_x + start_vec_y * end_vec_y).clamp(-1.0, 1.0);
    let mut delta_angle = dot.acos();

    if (start_vec_x * end_vec_y - start_vec_y * end_vec_x) < 0.0 {
        delta_angle = -delta_angle;
    }
    if sweep && delta_angle < 0.0 {
        delta_angle += 2.0 * pi;
    } else if !sweep && delta_angle > 0.0 {
        delta_angle -= 2.0 * pi;
    }

    let num_segments = (delta_angle.abs() / (pi / 2.0)).ceil() as usize;
    let mut beziers = Vec::with_capacity(num_segments);
    let angle_step = delta_angle / num_segments as f32;

    let mut current_angle = start_angle;

    for i in 0..num_segments {
        let next_angle = current_angle + angle_step;

        let alpha = (4.0/3.0) * (angle_step / 4.0).tan();

        let p0_unit = Point { x: current_angle.cos(), y: current_angle.sin() };
        let p3_unit = Point { x: next_angle.cos(), y: next_angle.sin() };

        let p1_unit = Point { x: p0_unit.x - alpha * p0_unit.y, y: p0_unit.y + alpha * p0_unit.x };
        let p2_unit = Point { x: p3_unit.x + alpha * p3_unit.y, y: p3_unit.y - alpha * p3_unit.x };

        let transform_unit_point = |p_unit: Point| -> Point {
            let x_scaled = p_unit.x * rx;
            let y_scaled = p_unit.y * ry;
            let x_rotated = cos_phi * x_scaled - sin_phi * y_scaled;
            let y_rotated = sin_phi * x_scaled + cos_phi * y_scaled;
            Point { x: x_rotated + cx, y: y_rotated + cy }
        };
        
        let p1 = transform_unit_point(p1_unit);
        let p2 = transform_unit_point(p2_unit);
        
        let p3 = if i == num_segments - 1 {
            end
        } else {
            transform_unit_point(p3_unit)
        };

        beziers.push(PathCommand::CubicBezier(p1, p2, p3));
        current_angle = next_angle;
    }
    beziers
}

pub(crate) fn apply_transform_to_path(commands: &[PathCommand], transform: &Transform) -> Vec<PathCommand> {
    let mut transformed_cmds = Vec::new();
    let mut current_pos = Point { x: 0.0, y: 0.0 };
    let mut subpath_start = Point { x: 0.0, y: 0.0 };

    for cmd in commands {
        match cmd.clone() {
            PathCommand::MoveTo(p) => {
                current_pos = p;
                subpath_start = p;
                transformed_cmds.push(PathCommand::MoveTo(transform.apply_point(p)));
            }
            PathCommand::LineTo(p) => {
                current_pos = p;
                transformed_cmds.push(PathCommand::LineTo(transform.apply_point(p)));
            }
            PathCommand::CubicBezier(cp1, cp2, end) => {
                current_pos = end;
                transformed_cmds.push(PathCommand::CubicBezier(
                    transform.apply_point(cp1),
                    transform.apply_point(cp2),
                    transform.apply_point(end),
                ));
            }
            PathCommand::QuadraticBezier(cp, end) => {
                current_pos = end;
                transformed_cmds.push(PathCommand::QuadraticBezier(
                    transform.apply_point(cp),
                    transform.apply_point(end),
                ));
            }
            PathCommand::Arc { rx, ry, x_axis_rotation, large_arc_flag, sweep_flag, end } => {
                let beziers = arc_to_beziers(current_pos, rx, ry, x_axis_rotation, large_arc_flag, sweep_flag, end);
                for bezier in beziers {
                    if let PathCommand::CubicBezier(cp1, cp2, end) = bezier {
                        transformed_cmds.push(PathCommand::CubicBezier(
                            transform.apply_point(cp1),
                            transform.apply_point(cp2),
                            transform.apply_point(end),
                        ));
                    }
                }
                current_pos = end;
            }
            PathCommand::ClosePath => {
                current_pos = subpath_start;
                transformed_cmds.push(PathCommand::ClosePath);
            }
        }
    }
    transformed_cmds
}

pub(crate) fn generate_color_map(
    bitmap: &[u8],
    paint: &Paint,
    width: usize,
    height: usize,
    draw_x: f32,
    draw_y: f32,
    bbox_x: f32,
    bbox_y: f32,
    bbox_w: f32,
    bbox_h: f32,
) -> Vec<u32> {
    let mut color_map = Vec::with_capacity(bitmap.len());

    match paint {
        Paint::Solid(color) => {
            let rgb = color & 0x00FFFFFF;
            let src_a = (color >> 24) & 0xFF;
            for &coverage in bitmap {
                let final_a = ((coverage as u32) * src_a) / 255;
                color_map.push((final_a << 24) | rgb);
            }
        }

        Paint::LinearGradient(_) | Paint::RadialGradient(_) => {
            for y in 0..height {
                for x in 0..width {
                    let idx = y * width + x;
                    let coverage = bitmap[idx];
                    let svg_x = draw_x + x as f32;
                    let svg_y = draw_y + y as f32;
                    let color = paint.get_color_at(svg_x, svg_y, bbox_x, bbox_y, bbox_w, bbox_h);
                    let src_a = ((color >> 24) & 0xFF) as u32;
                    let final_a = (src_a * (coverage as u32)) / 255;
                    color_map.push((final_a << 24) | (color & 0xFFFFFF));
                }
            }
        }

        Paint::None | Paint::Reference(_) => {
            for _ in 0..bitmap.len() {
                color_map.push(0x00000000);
            }
        }
    }

    color_map
}