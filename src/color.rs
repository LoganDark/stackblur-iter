use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct ARGB([i32; 4]);

impl ARGB {
	pub fn from_u32(argb: u32) -> Self {
		let [a, r, g, b] = argb.to_be_bytes();
		Self([a as i32, r as i32, g as i32, b as i32])
	}

	pub fn to_u32(self) -> u32 {
		let [a, r, g, b] = self.0;
		u32::from_be_bytes([a as u8, r as u8, g as u8, b as u8])
	}

	#[cfg(feature = "blend-srgb")]
	pub fn from_u32_srgb(argb: u32) -> Self {
		use blend_srgb::convert::srgb8_to_rgb12 as cvt;

		let [a, r, g, b] = argb.to_be_bytes();

		Self([cvt(a) as i32, cvt(r) as i32, cvt(g) as i32, cvt(b) as i32])
	}

	#[cfg(feature = "blend-srgb")]
	pub fn to_u32_srgb(self) -> u32 {
		use blend_srgb::convert::rgb12_to_srgb8 as cvt;

		let [a, r, g, b] = self.0;
		let [a, r, g, b] = [a as u16, r as u16, g as u16, b as u16];

		u32::from_be_bytes([cvt(a) as u8, cvt(r) as u8, cvt(g) as u8, cvt(b) as u8])
	}
}

impl Add for ARGB {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		let [a1, r1, g1, b1] = self.0;
		let [a2, r2, g2, b2] = rhs.0;
		Self([a1 + a2, r1 + r2, g1 + g2, b1 + b2])
	}
}

impl Sub for ARGB {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		let [a1, r1, g1, b1] = self.0;
		let [a2, r2, g2, b2] = rhs.0;
		Self([a1 - a2, r1 - r2, g1 - g2, b1 - b2])
	}
}

impl AddAssign for ARGB {
	fn add_assign(&mut self, rhs: Self) {
		self.0[0] += rhs.0[0];
		self.0[1] += rhs.0[1];
		self.0[2] += rhs.0[2];
		self.0[3] += rhs.0[3];
	}
}

impl SubAssign for ARGB {
	fn sub_assign(&mut self, rhs: Self) {
		self.0[0] -= rhs.0[0];
		self.0[1] -= rhs.0[1];
		self.0[2] -= rhs.0[2];
		self.0[3] -= rhs.0[3];
	}
}

impl Mul<usize> for ARGB {
	type Output = Self;

	fn mul(self, rhs: usize) -> Self::Output {
		let [a, r, g, b] = self.0;
		let rhs = rhs as i32;
		Self([a * rhs, r * rhs, g * rhs, b * rhs])
	}
}

impl Div<usize> for ARGB {
	type Output = Self;

	fn div(self, rhs: usize) -> Self::Output {
		let [a, r, g, b] = self.0;
		let rhs = rhs as i32;
		Self([a / rhs, r / rhs, g / rhs, b / rhs])
	}
}
