pub mod simd;

use crate::png::chunks::IhdrChunk;
use crate::png::chunks::ColorType;

pub fn unfilter(data: &[u8], ihdr: &IhdrChunk) -> Result<Vec<u8>, String> {
    let bpp = calculate_bpp(ihdr.color_type, ihdr.bit_depth)?;
    let width = ihdr.width as usize;
    let height = ihdr.height as usize;

    let bits_per_row = width * (ihdr.bit_depth as usize) * channels_per_pixel(ihdr.color_type);
    let bytes_per_row = (bits_per_row + 7) / 8;
    let stride = bytes_per_row + 1;

    if data.len() < stride * height {
        return Err(format!("Not enough data: expected {} bytes, got {}", stride * height, data.len()));
    }

    let mut unfiltered = Vec::with_capacity(bytes_per_row * height);
    let mut prev_row = vec![0u8; bytes_per_row];
    let mut current_row = vec![0u8; bytes_per_row];

    for row_idx in 0..height {
        let start = row_idx * stride;
        let filter_type = data[start];
        let scanline = &data[start + 1 .. start + stride];

        // Copy filtered data to current_row to work on it
        current_row.copy_from_slice(scanline);

        // Apply Inverse Filter
        match filter_type {
            0 => {}, // None: Do nothing
            1 => unfilter_sub(&mut current_row, bpp),
            2 => simd::unfilter_up(&mut current_row, &prev_row), // SIMD Optimized
            3 => unfilter_average(&mut current_row, &prev_row, bpp),
            4 => unfilter_paeth(&mut current_row, &prev_row, bpp),
            _ => return Err(format!("Unknown filter type: {}", filter_type)),
        }

        unfiltered.extend_from_slice(&current_row);
        prev_row.copy_from_slice(&current_row);
    }

    Ok(unfiltered)
}

fn unfilter_sub(row: &mut [u8], bpp: usize) {
    for i in bpp..row.len() {
        let a = row[i - bpp];
        row[i] = row[i].wrapping_add(a);
    }
}

fn unfilter_average(row: &mut [u8], prev: &[u8], bpp: usize) {
    for i in 0..row.len() {
        let a = if i >= bpp { row[i - bpp] } else { 0 };
        let b = prev[i];
        row[i] = row[i].wrapping_add(((a as u16 + b as u16) / 2) as u8);
    }
}

fn unfilter_paeth(row: &mut [u8], prev: &[u8], bpp: usize) {
    for i in 0..row.len() {
        let a = if i >= bpp { row[i - bpp] } else { 0 };
        let b = prev[i];
        let c = if i >= bpp { prev[i - bpp] } else { 0 };
        row[i] = row[i].wrapping_add(paeth_predictor(a, b, c));
    }
}

#[inline]
fn paeth_predictor(a: u8, b: u8, c: u8) -> u8 {
    let a = a as i16;
    let b = b as i16;
    let c = c as i16;

    let p = a + b - c;
    let pa = (p - a).abs();
    let pb = (p - b).abs();
    let pc = (p - c).abs();

    if pa <= pb && pa <= pc {
        a as u8
    } else if pb <= pc {
        b as u8
    } else {
        c as u8
    }
}

fn calculate_bpp(color_type: ColorType, bit_depth: u8) -> Result<usize, String> {
    let channels = channels_per_pixel(color_type);
    let bits_per_pixel = channels * (bit_depth as usize);
    Ok((bits_per_pixel + 7) / 8)
}

fn channels_per_pixel(color_type: ColorType) -> usize {
    match color_type {
        ColorType::Grayscale => 1,
        ColorType::RGB => 3,
        ColorType::Indexed => 1,
        ColorType::GrayscaleAlpha => 2,
        ColorType::RGBA => 4,
    }
}