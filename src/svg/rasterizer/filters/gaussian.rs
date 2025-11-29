use crate::svg::parser::tags::Tag;
use std::f32::consts::PI;

pub fn apply(input: &[u32], width: usize, height: usize, tag: &Tag) -> Vec<u32> {
    let std_dev_str = tag.params.get("stdDeviation").map(|s| s.as_str()).unwrap_or("0");
    let std_devs: Vec<f32> = std_dev_str
        .split_whitespace()
        .filter_map(|s| s.parse().ok())
        .collect();

    let (sigma_x, sigma_y) = if std_devs.is_empty() {
        (0.0, 0.0)
    } else if std_devs.len() == 1 {
        (std_devs[0], std_devs[0])
    } else {
        (std_devs[0], std_devs[1])
    };

    if sigma_x == 0.0 && sigma_y == 0.0 {
        return input.to_vec();
    }

    // Horizontal Pass
    let temp = if sigma_x > 0.0 {
        box_blur(input, width, height, sigma_x, true)
    } else {
        input.to_vec()
    };

    // Vertical Pass
    if sigma_y > 0.0 {
        box_blur(&temp, width, height, sigma_y, false)
    } else {
        temp
    }
}

// Using 3-pass Box Blur to approximate Gaussian
// http://blog.ivank.net/fastest-gaussian-blur.html
fn box_blur(src: &[u32], w: usize, h: usize, sigma: f32, horizontal: bool) -> Vec<u32> {
    let mut boxes = boxes_for_gauss(sigma, 3);
    let mut out = src.to_vec();
    let mut temp = src.to_vec();

    for &b in &boxes {
        let r = (b - 1) / 2;
        if horizontal {
            box_blur_h(&out, &mut temp, w, h, r);
        } else {
            box_blur_t(&out, &mut temp, w, h, r);
        }
        out.copy_from_slice(&temp);
    }
    out
}

fn boxes_for_gauss(sigma: f32, n: usize) -> Vec<isize> {
    let w_ideal = (12.0 * sigma * sigma / n as f32 + 1.0).sqrt();
    let mut wl = w_ideal.floor() as isize;
    if wl % 2 == 0 { wl -= 1; }
    let wu = wl + 2;

    let m_ideal = (12.0 * sigma * sigma - n as f32 * wl as f32 * wl as f32 - 4.0 * n as f32 * wl as f32 - 3.0 * n as f32) / (-4.0 * wl as f32 - 4.0);
    let m = m_ideal.round() as usize;

    let mut sizes = Vec::new();
    for i in 0..n {
        sizes.push(if i < m { wl } else { wu });
    }
    sizes
}

fn box_blur_h(scl: &[u32], tcl: &mut [u32], w: usize, h: usize, r: isize) {
    let iarr = 1.0 / (r + r + 1) as f32;
    
    for i in 0..h {
        let ti = i * w;
        let li = ti;
        let ri = ti + w - 1;
        
        let fv = scl[ti];
        let lv = scl[ri];
        
        let (mut val_a, mut val_r, mut val_g, mut val_b) = unpack(fv);
        
        val_a *= (r + 1) as f32;
        val_r *= (r + 1) as f32;
        val_g *= (r + 1) as f32;
        val_b *= (r + 1) as f32;

        for j in 0..r {
            let (a, red, g, b) = unpack(scl[ti + j as usize]);
            val_a += a; val_r += red; val_g += g; val_b += b;
        }
        
        for j in 0..=r {
            let (a, red, g, b) = unpack(scl[ri.min(ti + (j as usize))]);
            val_a += a; val_r += red; val_g += g; val_b += b;
        }
        
        // Initial Window setup above is slightly wrong for generic box blur, 
        // standard impl involves moving window.
        // Let's switch to a simpler accumulating sliding window.
        
        // Reset
        let mut val_a = 0.0; let mut val_r = 0.0; let mut val_g = 0.0; let mut val_b = 0.0;

        // Pre-fill
        for j in -r..=r {
            let idx = ti + (j.clamp(0, (w - 1) as isize) as usize);
            let (a, red, g, b) = unpack(scl[idx]);
            val_a += a; val_r += red; val_g += g; val_b += b;
        }

        for j in 0..w {
            tcl[ti + j] = pack(val_a * iarr, val_r * iarr, val_g * iarr, val_b * iarr);

            let p_out_idx = ti + ((j as isize - r).clamp(0, (w - 1) as isize) as usize);
            let p_in_idx = ti + ((j as isize + r + 1).clamp(0, (w - 1) as isize) as usize);

            let (out_a, out_r, out_g, out_b) = unpack(scl[p_out_idx]);
            let (in_a, in_r, in_g, in_b) = unpack(scl[p_in_idx]);

            val_a = val_a - out_a + in_a;
            val_r = val_r - out_r + in_r;
            val_g = val_g - out_g + in_g;
            val_b = val_b - out_b + in_b;
        }
    }
}

fn box_blur_t(scl: &[u32], tcl: &mut [u32], w: usize, h: usize, r: isize) {
    let iarr = 1.0 / (r + r + 1) as f32;

    for i in 0..w {
        let ti = i;
        let li = ti;
        let ri = ti + (h - 1) * w;

        let mut val_a = 0.0; let mut val_r = 0.0; let mut val_g = 0.0; let mut val_b = 0.0;

         // Pre-fill
         for j in -r..=r {
            let y = j.clamp(0, (h - 1) as isize) as usize;
            let idx = ti + y * w;
            let (a, red, g, b) = unpack(scl[idx]);
            val_a += a; val_r += red; val_g += g; val_b += b;
        }

        for j in 0..h {
            tcl[ti + j * w] = pack(val_a * iarr, val_r * iarr, val_g * iarr, val_b * iarr);

            let y_out = (j as isize - r).clamp(0, (h - 1) as isize) as usize;
            let y_in = (j as isize + r + 1).clamp(0, (h - 1) as isize) as usize;

            let p_out_idx = ti + y_out * w;
            let p_in_idx = ti + y_in * w;

            let (out_a, out_r, out_g, out_b) = unpack(scl[p_out_idx]);
            let (in_a, in_r, in_g, in_b) = unpack(scl[p_in_idx]);

            val_a = val_a - out_a + in_a;
            val_r = val_r - out_r + in_r;
            val_g = val_g - out_g + in_g;
            val_b = val_b - out_b + in_b;
        }
    }
}

#[inline]
fn unpack(c: u32) -> (f32, f32, f32, f32) {
    (
        ((c >> 24) & 0xFF) as f32,
        ((c >> 16) & 0xFF) as f32,
        ((c >> 8) & 0xFF) as f32,
        (c & 0xFF) as f32,
    )
}

#[inline]
fn pack(a: f32, r: f32, g: f32, b: f32) -> u32 {
    let ia = a.round().clamp(0.0, 255.0) as u32;
    let ir = r.round().clamp(0.0, 255.0) as u32;
    let ig = g.round().clamp(0.0, 255.0) as u32;
    let ib = b.round().clamp(0.0, 255.0) as u32;
    (ia << 24) | (ir << 16) | (ig << 8) | ib
}
