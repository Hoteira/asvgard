
#[cfg(target_arch = "x86_64")]
use core::arch::x86_64::*;

#[cfg(target_arch = "aarch64")]
use core::arch::aarch64::*;


pub fn unfilter_up(current: &mut [u8], prev: &[u8]) {

    #[cfg(target_arch = "x86_64")]
    pub fn unfilter_up(current: &mut [u8], prev: &[u8]) {

        #[cfg(not(feature = "std"))] // This condition activates when the "std" feature is OFF
        {
            unsafe {
                unfilter_up_sse2(current, prev);
                return;
            }
        }
    }

    #[cfg(target_arch = "aarch64")]
    {
        unsafe {
            unfilter_up_neon(current, prev);
            return; // Return after calling if SIMD was used
        }
    }

    unfilter_up_scalar(current, prev);
}

fn unfilter_up_scalar(current: &mut [u8], prev: &[u8]) {
    for (c, p) in current.iter_mut().zip(prev.iter()) {
        *c = c.wrapping_add(*p);
    }
}

#[cfg(target_arch = "x86_64")]
#[target_feature(enable = "sse2")]
unsafe fn unfilter_up_sse2(current: &mut [u8], prev: &[u8]) {
    let len = current.len();
    let mut i = 0;

    while i + 16 <= len {
        let curr_ptr = unsafe { current.as_mut_ptr().add(i) };
        let prev_ptr = unsafe { prev.as_ptr().add(i) };

        let curr_vec = unsafe { _mm_loadu_si128(curr_ptr as *const __m128i) };
        let prev_vec = unsafe { _mm_loadu_si128(prev_ptr as *const __m128i) };

        let result = _mm_add_epi8(curr_vec, prev_vec);

        unsafe { _mm_storeu_si128(curr_ptr as *mut __m128i, result) };
        i += 16;
    }

    unfilter_up_scalar(&mut current[i..], &prev[i..]);
}


#[cfg(target_arch = "aarch64")]
unsafe fn unfilter_up_neon(current: &mut [u8], prev: &[u8]) {
    let len = current.len();
    let mut i = 0;

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
