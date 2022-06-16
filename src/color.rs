use std::num::Wrapping;
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct Argb([Wrapping<u32>; 4]);

impl Argb {
	pub const fn from_u32(argb: u32) -> Self {
		let [a, r, g, b] = argb.to_be_bytes();
		Self([Wrapping(a as u32), Wrapping(r as u32), Wrapping(g as u32), Wrapping(b as u32)])
	}

	pub const fn to_u32(self) -> u32 {
		let [a, r, g, b] = self.0;
		u32::from_be_bytes([a.0 as u8, r.0 as u8, g.0 as u8, b.0 as u8])
	}

	#[cfg(feature = "blend-srgb")]
	pub fn from_u32_srgb(argb: u32) -> Self {
		use blend_srgb::convert::srgb8_to_rgb12 as cvt;

		let [a, r, g, b] = argb.to_be_bytes();

		Self([Wrapping(cvt(a) as u32), Wrapping(cvt(r) as u32), Wrapping(cvt(g) as u32), Wrapping(cvt(b) as u32)])
	}

	#[cfg(feature = "blend-srgb")]
	pub fn to_u32_srgb(self) -> u32 {
		use blend_srgb::convert::rgb12_to_srgb8 as cvt;

		let [a, r, g, b] = self.0;
		let [a, r, g, b] = [a.0 as u16, r.0 as u16, g.0 as u16, b.0 as u16];

		u32::from_be_bytes([cvt(a) as u8, cvt(r) as u8, cvt(g) as u8, cvt(b) as u8])
	}
}

impl Add for Argb {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		let [a1, r1, g1, b1] = self.0;
		let [a2, r2, g2, b2] = rhs.0;
		Self([a1 + a2, r1 + r2, g1 + g2, b1 + b2])
	}
}

impl Sub for Argb {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		let [a1, r1, g1, b1] = self.0;
		let [a2, r2, g2, b2] = rhs.0;
		Self([a1 - a2, r1 - r2, g1 - g2, b1 - b2])
	}
}

impl AddAssign for Argb {
	fn add_assign(&mut self, rhs: Self) {
		self.0[0] += rhs.0[0];
		self.0[1] += rhs.0[1];
		self.0[2] += rhs.0[2];
		self.0[3] += rhs.0[3];
	}
}

impl SubAssign for Argb {
	fn sub_assign(&mut self, rhs: Self) {
		self.0[0] -= rhs.0[0];
		self.0[1] -= rhs.0[1];
		self.0[2] -= rhs.0[2];
		self.0[3] -= rhs.0[3];
	}
}

impl Mul<usize> for Argb {
	type Output = Self;

	fn mul(self, rhs: usize) -> Self::Output {
		let [a, r, g, b] = self.0;
		let rhs = Wrapping(rhs as u32);
		Self([a * rhs, r * rhs, g * rhs, b * rhs])
	}
}

impl Div<usize> for Argb {
	type Output = Self;

	fn div(self, rhs: usize) -> Self::Output {
		let [a, r, g, b] = self.0;
		let rhs = Wrapping(rhs as u32);
		Self([a / rhs, r / rhs, g / rhs, b / rhs])
	}
}
