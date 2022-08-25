#![allow(unused_parens)]
#![allow(non_snake_case)]
#![allow(non_camel_case_types)]
#![allow(dead_code)]
#![allow(unused_variables)]
#![allow(unused_mut)]
#![allow(non_upper_case_globals)]

use faust_types::*;

<<includeIntrinsic>>
<<includeclass>>

// Potentially improves the performance of SIMD floating-point math
// by flushing denormals/underflow to zero.
// See: https://gist.github.com/GabrielMajeri/545042ee4f956d5b2141105eb6a505a9
// See: https://github.com/grame-cncm/faust/blob/master-dev/architecture/faust/dsp/dsp.h#L236
pub fn enable_flush_denormals_to_zero() {
    unsafe {
        use std::arch::x86_64::*;

        let mut mxcsr = _mm_getcsr();

        // Denormals & underflows are flushed to zero
        mxcsr |= (1 << 15) | (1 << 6);

        // All exceptions are masked
        mxcsr |= ((1 << 6) - 1) << 7;

        _mm_setcsr(mxcsr);
    }
}
