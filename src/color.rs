use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};
use std::simd::i32x4;

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct ARGB(i32x4);

impl ARGB {
	pub fn from_u32(argb: u32) -> Self {
		Self(i32x4::from_array(argb.to_be_bytes().map(|i| i as i32)))
	}

	pub fn to_u32(self) -> u32 {
		u32::from_be_bytes(self.0.to_array().map(|i| i as u8))
	}

	#[cfg(feature = "blend-srgb")]
	pub fn from_u32_srgb(argb: u32) -> Self {
		Self(i32x4::from_array(argb.to_be_bytes().map(|i| blend_srgb::convert::srgb8_to_rgb12(i) as i32)))
	}

	#[cfg(feature = "blend-srgb")]
	pub fn to_u32_srgb(self) -> u32 {
		u32::from_be_bytes(self.0.to_array().map(|i| blend_srgb::convert::rgb12_to_srgb8(i as u16) as u8))
	}
}

impl Add for ARGB {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self(self.0 + rhs.0)
	}
}

impl Sub for ARGB {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Self(self.0 - rhs.0)
	}
}

impl AddAssign for ARGB {
	fn add_assign(&mut self, rhs: Self) {
		*self = *self + rhs;
	}
}

impl SubAssign for ARGB {
	fn sub_assign(&mut self, rhs: Self) {
		*self = *self - rhs;
	}
}

impl Mul<usize> for ARGB {
	type Output = Self;

	fn mul(self, rhs: usize) -> Self::Output {
		Self(self.0 * i32x4::splat(rhs as i32))
	}
}

impl Div<usize> for ARGB {
	type Output = Self;

	fn div(self, rhs: usize) -> Self::Output {
		let [a, r, g, b] = self.0.to_array();
		Self(i32x4::from_array([a / rhs as i32, r / rhs as i32, g / rhs as i32, b / rhs as i32]))
	}
}
