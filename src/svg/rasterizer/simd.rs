#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

/// Blends a slice of source pixels into a destination slice using standard source-over composition.
/// Assumes pixels are ARGB (u32).
pub fn blend_scanline(dst: &mut [u32], src: &[u32]) {
    let len = dst.len().min(src.len());
    if len == 0 {
        return;
    }

    #[cfg(all(target_arch = "x86_64", target_feature = "sse2"))]
    {
        if is_x86_feature_detected!("sse2") {
            unsafe {
                blend_scanline_sse2(dst, src, len);
                return;
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        unsafe {
            blend_scanline_neon(dst, src, len);
            return;
        }
    }

    blend_scanline_scalar(dst, src, len);
}

fn blend_scanline_scalar(dst: &mut [u32], src: &[u32], len: usize) {
    for i in 0..len {
        let s = src[i];
        let d = dst[i];
        dst[i] = blend_pixel(d, s);
    }
}

#[inline(always)]
fn blend_pixel(dst: u32, src: u32) -> u32 {
    let src_a = ((src >> 24) & 0xFF) as f32 / 255.0;
    let src_r = ((src >> 16) & 0xFF) as f32;
    let src_g = ((src >> 8) & 0xFF) as f32;
    let src_b = (src & 0xFF) as f32;

    let dst_a = ((dst >> 24) & 0xFF) as f32 / 255.0;
    let dst_r = ((dst >> 16) & 0xFF) as f32;
    let dst_g = ((dst >> 8) & 0xFF) as f32;
    let dst_b = (dst & 0xFF) as f32;

    let out_a = src_a + dst_a * (1.0 - src_a);
    let safe_a = out_a.max(0.001);

    let out_r = (src_r * src_a + dst_r * dst_a * (1.0 - src_a)) / safe_a;
    let out_g = (src_g * src_a + dst_g * dst_a * (1.0 - src_a)) / safe_a;
    let out_b = (src_b * src_a + dst_b * dst_a * (1.0 - src_a)) / safe_a;

    let a = (out_a.clamp(0.0, 1.0) * 255.0) as u32;
    let r = out_r.clamp(0.0, 255.0) as u32;
    let g = out_g.clamp(0.0, 255.0) as u32;
    let b = out_b.clamp(0.0, 255.0) as u32;

    (a << 24) | (r << 16) | (g << 8) | b
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
unsafe fn blend_scanline_sse2(dst: &mut [u32], src: &[u32], len: usize) {
    let mut i = 0;
    
    let scale_inv_255 = _mm_set1_ps(1.0 / 255.0);
    let scale_255 = _mm_set1_ps(255.0);
    let ones = _mm_set1_ps(1.0);
    let epsilon = _mm_set1_ps(0.001);
    let zeros = _mm_set1_ps(0.0);

    while i + 4 <= len {
        // Pointer arithmetic is unsafe
        let s_ptr = unsafe { src.as_ptr().add(i) };
        let d_ptr = unsafe { dst.as_mut_ptr().add(i) };

        // Load 4 pixels (128 bits)
        let s_int = unsafe { _mm_loadu_si128(s_ptr as *const __m128i) };
        let d_int = unsafe { _mm_loadu_si128(d_ptr as *const __m128i) };

        let zero = _mm_setzero_si128();
        let s_lo = _mm_unpacklo_epi8(s_int, zero);
        let s_hi = _mm_unpackhi_epi8(s_int, zero);
        let d_lo = _mm_unpacklo_epi8(d_int, zero);
        let d_hi = _mm_unpackhi_epi8(d_int, zero);

        let s_p0 = _mm_unpacklo_epi16(s_lo, zero);
        let s_p1 = _mm_unpackhi_epi16(s_lo, zero);
        let s_p2 = _mm_unpacklo_epi16(s_hi, zero);
        let s_p3 = _mm_unpackhi_epi16(s_hi, zero);

        let d_p0 = _mm_unpacklo_epi16(d_lo, zero);
        let d_p1 = _mm_unpackhi_epi16(d_lo, zero);
        let d_p2 = _mm_unpacklo_epi16(d_hi, zero);
        let d_p3 = _mm_unpackhi_epi16(d_hi, zero);

        let mut s_f0 = _mm_cvtepi32_ps(s_p0);
        let mut s_f1 = _mm_cvtepi32_ps(s_p1);
        let mut s_f2 = _mm_cvtepi32_ps(s_p2);
        let mut s_f3 = _mm_cvtepi32_ps(s_p3);

        let mut d_f0 = _mm_cvtepi32_ps(d_p0);
        let mut d_f1 = _mm_cvtepi32_ps(d_p1);
        let mut d_f2 = _mm_cvtepi32_ps(d_p2);
        let mut d_f3 = _mm_cvtepi32_ps(d_p3);

        _MM_TRANSPOSE4_PS(&mut s_f0, &mut s_f1, &mut s_f2, &mut s_f3);
        _MM_TRANSPOSE4_PS(&mut d_f0, &mut d_f1, &mut d_f2, &mut d_f3);

        let src_b = s_f0;
        let src_g = s_f1;
        let src_r = s_f2;
        let src_a_byte = s_f3;

        let dst_b = d_f0;
        let dst_g = d_f1;
        let dst_r = d_f2;
        let dst_a_byte = d_f3;

        let src_a = _mm_mul_ps(src_a_byte, scale_inv_255);
        let dst_a = _mm_mul_ps(dst_a_byte, scale_inv_255);

        let one_minus_src_a = _mm_sub_ps(ones, src_a);
        let out_a = _mm_add_ps(src_a, _mm_mul_ps(dst_a, one_minus_src_a));
        let safe_out_a = _mm_max_ps(out_a, epsilon);

        let term2_weight = _mm_mul_ps(dst_a, one_minus_src_a);
        
        let out_r_num = _mm_add_ps(_mm_mul_ps(src_r, src_a), _mm_mul_ps(dst_r, term2_weight));
        let out_g_num = _mm_add_ps(_mm_mul_ps(src_g, src_a), _mm_mul_ps(dst_g, term2_weight));
        let out_b_num = _mm_add_ps(_mm_mul_ps(src_b, src_a), _mm_mul_ps(dst_b, term2_weight));

        let out_r = _mm_div_ps(out_r_num, safe_out_a);
        let out_g = _mm_div_ps(out_g_num, safe_out_a);
        let out_b = _mm_div_ps(out_b_num, safe_out_a);

        let out_a_scaled = _mm_mul_ps(out_a, scale_255);
        
        let r_clamped = _mm_min_ps(_mm_max_ps(out_r, zeros), scale_255);
        let g_clamped = _mm_min_ps(_mm_max_ps(out_g, zeros), scale_255);
        let b_clamped = _mm_min_ps(_mm_max_ps(out_b, zeros), scale_255);
        let a_clamped = _mm_min_ps(_mm_max_ps(out_a_scaled, zeros), scale_255);

        let i_b = _mm_cvtps_epi32(b_clamped);
        let i_g = _mm_cvtps_epi32(g_clamped);
        let i_r = _mm_cvtps_epi32(r_clamped);
        let i_a = _mm_cvtps_epi32(a_clamped);

        let t0 = _mm_unpacklo_epi32(i_b, i_g);
        let t1 = _mm_unpackhi_epi32(i_b, i_g);
        let t2 = _mm_unpacklo_epi32(i_r, i_a);
        let t3 = _mm_unpackhi_epi32(i_r, i_a);

        let pixel0 = _mm_unpacklo_epi64(t0, t2);
        let pixel1 = _mm_unpackhi_epi64(t0, t2);
        let pixel2 = _mm_unpacklo_epi64(t1, t3);
        let pixel3 = _mm_unpackhi_epi64(t1, t3);
        
        let p01_16 = _mm_packs_epi32(pixel0, pixel1);
        let p23_16 = _mm_packs_epi32(pixel2, pixel3);
        
        let result = _mm_packus_epi16(p01_16, p23_16);
        
        unsafe { _mm_storeu_si128(d_ptr as *mut __m128i, result) };

        i += 4;
    }

    blend_scanline_scalar(&mut dst[i..], &src[i..], len - i);
}

#[cfg(target_arch = "aarch64")]
unsafe fn blend_scanline_neon(dst: &mut [u32], src: &[u32], len: usize) {
    let mut i = 0;
    
    let inv_255 = unsafe { vdupq_n_f32(1.0 / 255.0) };
    let ones = unsafe { vdupq_n_f32(1.0) };
    let epsilon = unsafe { vdupq_n_f32(0.001) };

    while i + 4 <= len {
        let s_ptr = unsafe { src.as_ptr().add(i) };
        let d_ptr = unsafe { dst.as_mut_ptr().add(i) };

        let s_raw = unsafe { vld1q_u8(s_ptr as *const u8) };
        let d_raw = unsafe { vld1q_u8(d_ptr as *const u8) };

        let s_low = unsafe { vmovl_u8(vget_low_u8(s_raw)) };
        let s_high = unsafe { vmovl_u8(vget_high_u8(s_raw)) };
        let d_low = unsafe { vmovl_u8(vget_low_u8(d_raw)) };
        let d_high = unsafe { vmovl_u8(vget_high_u8(d_raw)) };
        
        let s_p0 = unsafe { vmovl_u16(vget_low_u16(s_low)) };
        let s_p1 = unsafe { vmovl_u16(vget_high_u16(s_low)) };
        let s_p2 = unsafe { vmovl_u16(vget_low_u16(s_high)) };
        let s_p3 = unsafe { vmovl_u16(vget_high_u16(s_high)) };
        
        let d_p0 = unsafe { vmovl_u16(vget_low_u16(d_low)) };
        let d_p1 = unsafe { vmovl_u16(vget_high_u16(d_low)) };
        let d_p2 = unsafe { vmovl_u16(vget_low_u16(d_high)) };
        let d_p3 = unsafe { vmovl_u16(vget_high_u16(d_high)) };

        let mut s_f0 = unsafe { vcvtq_f32_u32(s_p0) };
        let mut s_f1 = unsafe { vcvtq_f32_u32(s_p1) };
        let mut s_f2 = unsafe { vcvtq_f32_u32(s_p2) };
        let mut s_f3 = unsafe { vcvtq_f32_u32(s_p3) };

        let mut d_f0 = unsafe { vcvtq_f32_u32(d_p0) };
        let mut d_f1 = unsafe { vcvtq_f32_u32(d_p1) };
        let mut d_f2 = unsafe { vcvtq_f32_u32(d_p2) };
        let mut d_f3 = unsafe { vcvtq_f32_u32(d_p3) };

        let uzp01 = unsafe { vuzpq_f32(s_f0, s_f1) };
        let uzp23 = unsafe { vuzpq_f32(s_f2, s_f3) };
        let br01 = uzp01.0;
        let ga01 = uzp01.1;
        let br23 = uzp23.0;
        let ga23 = uzp23.1;
        let uzp_b_r = unsafe { vuzpq_f32(br01, br23) };
        let uzp_g_a = unsafe { vuzpq_f32(ga01, ga23) };
        
        let src_b = uzp_b_r.0;
        let src_r = uzp_b_r.1;
        let src_g = uzp_g_a.0;
        let src_a_byte = uzp_g_a.1;

        let d_uzp01 = unsafe { vuzpq_f32(d_f0, d_f1) };
        let d_uzp23 = unsafe { vuzpq_f32(d_f2, d_f3) };
        let d_uzp_b_r = unsafe { vuzpq_f32(d_uzp01.0, d_uzp23.0) };
        let d_uzp_g_a = unsafe { vuzpq_f32(d_uzp01.1, d_uzp23.1) };
        let dst_b = d_uzp_b_r.0;
        let dst_r = d_uzp_b_r.1;
        let dst_g = d_uzp_g_a.0;
        let dst_a_byte = d_uzp_g_a.1;

        let src_a = unsafe { vmulq_f32(src_a_byte, inv_255) };
        let dst_a = unsafe { vmulq_f32(dst_a_byte, inv_255) };

        let one_minus_src_a = unsafe { vsubq_f32(ones, src_a) };
        let out_a = unsafe { vaddq_f32(src_a, vmulq_f32(dst_a, one_minus_src_a)) };
        let safe_out_a = unsafe { vmaxq_f32(out_a, epsilon) };
        
        let term2_weight = unsafe { vmulq_f32(dst_a, one_minus_src_a) };
        
        let out_r_num = unsafe { vaddq_f32(vmulq_f32(src_r, src_a), vmulq_f32(dst_r, term2_weight)) };
        let out_g_num = unsafe { vaddq_f32(vmulq_f32(src_g, src_a), vmulq_f32(dst_g, term2_weight)) };
        let out_b_num = unsafe { vaddq_f32(vmulq_f32(src_b, src_a), vmulq_f32(dst_b, term2_weight)) };
        
        let out_r = unsafe { vdivq_f32(out_r_num, safe_out_a) };
        let out_g = unsafe { vdivq_f32(out_g_num, safe_out_a) };
        let out_b = unsafe { vdivq_f32(out_b_num, safe_out_a) };
        
        let res_a = unsafe { vmulq_f32(out_a, vdupq_n_f32(255.0)) };
        let max_255 = unsafe { vdupq_n_f32(255.0) };
        let zero = unsafe { vdupq_n_f32(0.0) };
        
        let r_c = unsafe { vminq_f32(vmaxq_f32(out_r, zero), max_255) };
        let g_c = unsafe { vminq_f32(vmaxq_f32(out_g, zero), max_255) };
        let b_c = unsafe { vminq_f32(vmaxq_f32(out_b, zero), max_255) };
        let a_c = unsafe { vminq_f32(vmaxq_f32(res_a, zero), max_255) };

        let i_r = unsafe { vcvtq_u32_f32(r_c) };
        let i_g = unsafe { vcvtq_u32_f32(g_c) };
        let i_b = unsafe { vcvtq_u32_f32(b_c) };
        let i_a = unsafe { vcvtq_u32_f32(a_c) };

        let zip_br = unsafe { vzipq_u32(i_b, i_r) };
        let zip_ga = unsafe { vzipq_u32(i_g, i_a) };
        let res01 = unsafe { vzipq_u32(zip_br.0, zip_ga.0) }; 
        let res23 = unsafe { vzipq_u32(zip_br.1, zip_ga.1) };
        
        let p0_16 = unsafe { vqmovn_u32(res01.0) };
        let p1_16 = unsafe { vqmovn_u32(res01.1) };
        let p2_16 = unsafe { vqmovn_u32(res23.0) };
        let p3_16 = unsafe { vqmovn_u32(res23.1) };
        
        let p01_16 = unsafe { vcombine_u16(p0_16, p1_16) };
        let p23_16 = unsafe { vcombine_u16(p2_16, p3_16) };
        
        let p01_8 = unsafe { vqmovn_u16(p01_16) };
        let p23_8 = unsafe { vqmovn_u16(p23_16) };
        
        let final_vec = unsafe { vcombine_u8(p01_8, p23_8) };
        
        unsafe { vst1q_u8(d_ptr as *mut u8, final_vec) };

        i += 4;
    }
    
    blend_scanline_scalar(&mut dst[i..], &src[i..], len - i);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_blend_scanline_simd() {
        let len = 17; // Test odd length to trigger remainder
        let mut dst = vec![0xFF0000FF; len]; // Blue background (Opaque)
        let src = vec![0x80FF0000; len]; // Red with 50% alpha

        // Expected: Red on Blue.
        // src_a = 0.5
        // dst_a = 1.0
        // out_a = 0.5 + 1.0 * (1 - 0.5) = 1.0
        // out_r = (255 * 0.5 + 0 * 0.5 * 0.5) / 1.0 = 127.5 -> 128
        // out_g = 0
        // out_b = (0 * 0.5 + 255 * 1.0 * 0.5) / 1.0 = 127.5 -> 128
        
        // Expected color: FF 80 00 80 (approx)
        
        blend_scanline(&mut dst, &src);

        for i in 0..len {
            let d = dst[i];
            let a = (d >> 24) & 0xFF;
            let r = (d >> 16) & 0xFF;
            let g = (d >> 8) & 0xFF;
            let b = d & 0xFF;

            assert_eq!(a, 255, "Alpha mismatch at {}", i);
            assert!((r as i32 - 127).abs() <= 2, "Red mismatch at {}: got {}", i, r);
            assert_eq!(g, 0, "Green mismatch at {}", i);
            assert!((b as i32 - 127).abs() <= 2, "Blue mismatch at {}: got {}", i, b);
        }
    }
}
