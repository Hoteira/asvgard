#[cfg(feature = "std")]
pub use ::std::collections::{HashMap, VecDeque};

#[cfg(not(feature = "std"))]
pub use ::alloc::collections::BTreeMap as HashMap;
#[cfg(not(feature = "std"))]
pub use ::alloc::collections::VecDeque;

#[cfg(feature = "std")]
pub use ::std::vec::Vec;
#[cfg(not(feature = "std"))]
pub use ::alloc::vec::Vec;

#[cfg(feature = "std")]
pub use ::std::vec; 

#[cfg(not(feature = "std"))]
pub use ::alloc::vec; 

#[cfg(feature = "std")]
pub use ::std::string::{String, ToString};
#[cfg(not(feature = "std"))]
pub use ::alloc::string::{String, ToString};

#[cfg(feature = "std")]
pub use ::std::format;
#[cfg(not(feature = "std"))]
pub use ::alloc::format;

#[cfg(not(feature = "std"))]
use crate::utils::math;

pub trait FloatExt {
    fn sqrt(self) -> Self;
    fn sin(self) -> Self;
    fn cos(self) -> Self;
    fn round(self) -> Self;
    fn floor(self) -> Self;
    fn ceil(self) -> Self;
    fn abs(self) -> Self;
    fn powi(self, n: i32) -> Self;
    fn atan2(self, other: Self) -> Self;
    fn acos(self) -> Self;
    fn max(self, other: Self) -> Self;
    fn min(self, other: Self) -> Self;
    fn clamp(self, min: Self, max: Self) -> Self;
    fn signum(self) -> Self;
}

impl FloatExt for f32 {
    fn sqrt(self) -> Self {
        #[cfg(feature = "std")]
        return self.sqrt();
        #[cfg(not(feature = "std"))]
        return math::sqrt(self);
    }

    fn sin(self) -> Self {
        #[cfg(feature = "std")]
        return self.sin();
        #[cfg(not(feature = "std"))]
        return math::sin(self);
    }

    fn cos(self) -> Self {
        #[cfg(feature = "std")]
        return self.cos();
        #[cfg(not(feature = "std"))]
        return math::cos(self);
    }

    fn round(self) -> Self {
        #[cfg(feature = "std")]
        return self.round();
        #[cfg(not(feature = "std"))]
        return math::round(self);
    }

    fn floor(self) -> Self {
        #[cfg(feature = "std")]
        return self.floor();
        #[cfg(not(feature = "std"))]
        return math::floor(self);
    }

    fn ceil(self) -> Self {
        #[cfg(feature = "std")]
        return self.ceil();
        #[cfg(not(feature = "std"))]
        return math::ceil(self);
    }

    fn abs(self) -> Self {
        core::primitive::f32::abs(self)
    }

    fn powi(self, n: i32) -> Self {
        #[cfg(feature = "std")]
        return self.powi(n);
        #[cfg(not(feature = "std"))]
        return math::powi(self, n);
    }

    fn atan2(self, other: Self) -> Self {
        #[cfg(feature = "std")]
        return self.atan2(other);
        #[cfg(not(feature = "std"))]
        return math::atan2(self, other);
    }

    fn acos(self) -> Self {
        #[cfg(feature = "std")]
        return self.acos();
        #[cfg(not(feature = "std"))]
        return math::acos(self);
    }

    fn max(self, other: Self) -> Self {
        core::primitive::f32::max(self, other)
    }

    fn min(self, other: Self) -> Self {
        core::primitive::f32::min(self, other)
    }

    fn clamp(self, min: Self, max: Self) -> Self {
        core::primitive::f32::clamp(self, min, max)
    }

    fn signum(self) -> Self {
        if self > 0.0 { 1.0 } else if self < 0.0 { -1.0 } else { 0.0 }
    }
}