
#[cfg(target_arch = "x86_64")]
use std::arch::x86_64::*;

#[cfg(target_arch = "aarch64")]
use std::arch::aarch64::*;

/// Optimized implementation of the 'Up' filter: `x + b`.
/// This is perfectly parallelizable.
pub fn unfilter_up(current: &mut [u8], prev: &[u8]) {
    // x86_64 AVX2 Implementation (if explicit AVX2 functions were used)
    #[cfg(all(target_arch = "x86_64", target_feature = "avx2"))] // `target_feature` requires explicit enabling in Cargo.toml or rustflags
    {
        // Check for AVX2 at runtime
        if is_x86_feature_detected!("avx2") {
            unsafe {
                // If we had an AVX2 specific function, call it here
                // return unfilter_up_avx2(current, prev); 
            }
        }
    }
    
    // x86_64 SSE2 Implementation (Widely available)
    #[cfg(target_arch = "x86_64")]
    {
        // SSE2 is often enabled by default on x86_64, but checking is safest.
        if is_x86_feature_detected!("sse2") {
            unsafe {
                unfilter_up_sse2(current, prev);
                return; // Return after calling if SIMD was used
            }
        }
    }

    // AArch64 NEON Implementation
    #[cfg(target_arch = "aarch64")]
    {
        // NEON is typically always available on aarch64
        unsafe {
            unfilter_up_neon(current, prev);
            return; // Return after calling if SIMD was used
        }
    }

    // Fallback to scalar if no SIMD matches or platform not supported
    unfilter_up_scalar(current, prev);
}

fn unfilter_up_scalar(current: &mut [u8], prev: &[u8]) {
    for (c, p) in current.iter_mut().zip(prev.iter()) {
        *c = c.wrapping_add(*p);
    }
}

// --- x86_64 SSE2 ---
#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")] // Explicitly enable SSE2 for this function
unsafe fn unfilter_up_sse2(current: &mut [u8], prev: &[u8]) {
    let len = current.len();
    let mut i = 0;

    // Process 16 bytes at a time (128 bits)
    while i + 16 <= len {
        let curr_ptr = unsafe { current.as_mut_ptr().add(i) };
        let prev_ptr = unsafe { prev.as_ptr().add(i) };

        let curr_vec = unsafe { _mm_loadu_si128(curr_ptr as *const __m128i) };
        let prev_vec = unsafe { _mm_loadu_si128(prev_ptr as *const __m128i) };

        let result = _mm_add_epi8(curr_vec, prev_vec);

        unsafe { _mm_storeu_si128(curr_ptr as *mut __m128i, result) };
        i += 16;
    }

    // Handle remainder
    unfilter_up_scalar(&mut current[i..], &prev[i..]);
}

// --- AArch64 NEON ---
#[cfg(target_arch = "aarch64")]
unsafe fn unfilter_up_neon(current: &mut [u8], prev: &[u8]) {
    let len = current.len();
    let mut i = 0;

    // Process 16 bytes at a time (128 bits)
    while i + 16 <= len {
        let curr_ptr = unsafe { current.as_mut_ptr().add(i) };
        let prev_ptr = unsafe { prev.as_ptr().add(i) };

        let curr_vec = unsafe { vld1q_u8(curr_ptr) };
        let prev_vec = unsafe { vld1q_u8(prev_ptr) };

        let result = vaddq_u8(curr_vec, prev_vec);

        unsafe { vst1q_u8(curr_ptr, result) };
        i += 16;
    }

    unfilter_up_scalar(&mut current[i..], &prev[i..]);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unfilter_up_simd() {
        // Test SIMD-width processing (multiple of 16) + remainder
        let len = 35; 
        let mut current = vec![0u8; len];
        let mut prev = vec![0u8; len];

        // Initialize with some values to test wrapping
        for i in 0..len {
            current[i] = 10;
            prev[i] = 250; // 10 + 250 = 260 -> 4 (u8 wrapping)
        }

        unfilter_up(&mut current, &prev);

        for i in 0..len {
            assert_eq!(current[i], 4, "Mismatch at index {}", i);
        }
    }
}
