use crate::utils::compat::Vec;
use crate::utils::compat::vec;
use crate::utils::compat::FloatExt;

pub fn resize_image(original: &[u32], src_w: usize, src_h: usize, dst_w: usize, dst_h: usize) -> Vec<u32> {
    let mut resized = vec![0u32; dst_w * dst_h];

    // Bilinear Interpolation
    for y in 0..dst_h {
        for x in 0..dst_w {
            // Map target coordinate (x, y) to source coordinate (gx, gy)
            // We use center alignment: (x + 0.5) / dst_w = (gx + 0.5) / src_w
            let gx = ((x as f32 + 0.5) / dst_w as f32) * src_w as f32 - 0.5;
            let gy = ((y as f32 + 0.5) / dst_h as f32) * src_h as f32 - 0.5;

            let gxi = gx.floor() as isize;
            let gyi = gy.floor() as isize;

            let c00 = get_pixel_safe(original, src_w, src_h, gxi, gyi);
            let c10 = get_pixel_safe(original, src_w, src_h, gxi + 1, gyi);
            let c01 = get_pixel_safe(original, src_w, src_h, gxi, gyi + 1);
            let c11 = get_pixel_safe(original, src_w, src_h, gxi + 1, gyi + 1);

            let tx = gx - gxi as f32;
            let ty = gy - gyi as f32;

            // Interpolate
            let a = lerp_color(c00, c10, tx);
            let b = lerp_color(c01, c11, tx);
            let final_color = lerp_color(a, b, ty);

            resized[y * dst_w + x] = final_color;
        }
    }

    resized
}

#[inline]
fn get_pixel_safe(pixels: &[u32], w: usize, h: usize, x: isize, y: isize) -> u32 {
    let x = x.clamp(0, w as isize - 1) as usize;
    let y = y.clamp(0, h as isize - 1) as usize;
    pixels[y * w + x]
}

#[inline]
fn lerp_color(c1: u32, c2: u32, t: f32) -> u32 {
    let a1 = ((c1 >> 24) & 0xFF) as f32;
    let r1 = ((c1 >> 16) & 0xFF) as f32;
    let g1 = ((c1 >> 8) & 0xFF) as f32;
    let b1 = (c1 & 0xFF) as f32;

    let a2 = ((c2 >> 24) & 0xFF) as f32;
    let r2 = ((c2 >> 16) & 0xFF) as f32;
    let g2 = ((c2 >> 8) & 0xFF) as f32;
    let b2 = (c2 & 0xFF) as f32;

    let a = (a1 + (a2 - a1) * t) as u32;
    let r = (r1 + (r2 - r1) * t) as u32;
    let g = (g1 + (g2 - g1) * t) as u32;
    let b = (b1 + (b2 - b1) * t) as u32;

    (a << 24) | (r << 16) | (g << 8) | b
}
