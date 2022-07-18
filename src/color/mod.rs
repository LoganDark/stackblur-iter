use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};
use crate::StackBlurrable;

pub mod serial;
#[cfg(feature = "simd")]
pub mod simd;

use serial::StackBlurrableU32;
#[cfg(feature = "simd")]
use simd::StackBlurrableU32xN;

#[repr(transparent)]
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub struct Argb<T: StackBlurrable, const N: usize>(pub(crate) [T; N]);

impl Argb<StackBlurrableU32, 4> {
	pub fn from_u32(argb: u32) -> Self {
		let [a, r, g, b] = argb.to_be_bytes();
		let cvt = |i| StackBlurrableU32(i as u32);
		Self([cvt(a), cvt(r), cvt(g), cvt(b)])
	}

	pub fn to_u32(self) -> u32 {
		let [a, r, g, b] = self.0;
		let cvt = |i: StackBlurrableU32| i.0 as u8;
		u32::from_be_bytes([cvt(a), cvt(r), cvt(g), cvt(b)])
	}

	#[cfg(feature = "blend-srgb")]
	pub fn from_u32_srgb(argb: u32) -> Self {
		use blend_srgb::convert::srgb8_to_rgb12;

		let [a, r, g, b] = argb.to_be_bytes();
		let cvt = |i| StackBlurrableU32(srgb8_to_rgb12(i) as u32);
		Self([StackBlurrableU32(a as u32), cvt(r), cvt(g), cvt(b)])
	}

	#[cfg(feature = "blend-srgb")]
	pub fn to_u32_srgb(self) -> u32 {
		use blend_srgb::convert::rgb12_to_srgb8;

		let [a, r, g, b] = self.0;
		let [a, r, g, b] = [a.0 as u16, r.0 as u16, g.0 as u16, b.0 as u16];
		let cvt = |i| rgb12_to_srgb8(i) as u8;
		u32::from_be_bytes([a as u8, cvt(r), cvt(g), cvt(b)])
	}
}

#[allow(non_snake_case)]
#[cfg(feature = "simd")]
impl<const N: usize> Argb<StackBlurrableU32xN<N>, 4> where simd::LaneCount<N>: simd::SupportedLaneCount {
	pub fn from_u32xN(pixels: [u32; N]) -> Self {
		let arrs: [[u8; 4]; N] = pixels.map(u32::to_be_bytes);
		let a = simd::Simd::<u32, N>::from_array(arrs.map(|a| a[0] as u32));
		let r = simd::Simd::<u32, N>::from_array(arrs.map(|a| a[1] as u32));
		let g = simd::Simd::<u32, N>::from_array(arrs.map(|a| a[2] as u32));
		let b = simd::Simd::<u32, N>::from_array(arrs.map(|a| a[3] as u32));
		let cvt = StackBlurrableU32xN::<N>;
		Self([cvt(a), cvt(r), cvt(g), cvt(b)])
	}

	pub fn to_u32xN(self) -> [u32; N] {
		let [a, r, g, b] = self.0.map(|i| i.0.to_array());

		let mut countup = 0usize..;
		[(); N].map(move |_| {
			let i = countup.next().unwrap();
			u32::from_be_bytes([a[i] as u8, r[i] as u8, g[i] as u8, b[i] as u8])
		})
	}

	#[cfg(feature = "blend-srgb")]
	pub fn from_u32xN_srgb(pixels: [u32; N]) -> Self {
		use blend_srgb::convert::srgb8_to_rgb12;
		let arrs: [[u8; 4]; N] = pixels.map(u32::to_be_bytes);
		let a = simd::Simd::<u32, N>::from_array(arrs.map(|a| a[0] as u32));
		let r = simd::Simd::<u32, N>::from_array(arrs.map(|a| srgb8_to_rgb12(a[1]) as u32));
		let g = simd::Simd::<u32, N>::from_array(arrs.map(|a| srgb8_to_rgb12(a[2]) as u32));
		let b = simd::Simd::<u32, N>::from_array(arrs.map(|a| srgb8_to_rgb12(a[3]) as u32));
		let cvt = StackBlurrableU32xN;
		Self([cvt(a), cvt(r), cvt(g), cvt(b)])
	}

	#[cfg(feature = "blend-srgb")]
	pub fn to_u32xN_srgb(self) -> [u32; N] {
		use blend_srgb::convert::rgb12_to_srgb8;
		let [a, r, g, b] = self.0.map(|i| i.0.to_array());

		let mut countup = 0usize..;
		[(); N].map(move |_| {
			let i = countup.next().unwrap();
			u32::from_be_bytes([
				a[i] as u8,
				rgb12_to_srgb8(r[i] as u16),
				rgb12_to_srgb8(g[i] as u16),
				rgb12_to_srgb8(b[i] as u16)
			])
		})
	}
}

impl<T: StackBlurrable, const N: usize> Default for Argb<T, N> {
	fn default() -> Self {
		Self([(); N].map(|_| T::default()))
	}
}

impl<T: StackBlurrable, const N: usize> Add for Argb<T, N> {
	type Output = Self;

	fn add(mut self, rhs: Self) -> Self::Output {
		self += rhs;
		self
	}
}

impl<T: StackBlurrable, const N: usize> Sub for Argb<T, N> {
	type Output = Self;

	fn sub(mut self, rhs: Self) -> Self::Output {
		self -= rhs;
		self
	}
}

impl<T: StackBlurrable, const N: usize> AddAssign for Argb<T, N> {
	fn add_assign(&mut self, rhs: Self) {
		for (lhs, rhs) in self.0.iter_mut().zip(rhs.0.into_iter()) {
			*lhs += rhs;
		}
	}
}

impl<T: StackBlurrable, const N: usize> SubAssign for Argb<T, N> {
	fn sub_assign(&mut self, rhs: Self) {
		for (lhs, rhs) in self.0.iter_mut().zip(rhs.0.into_iter()) {
			*lhs -= rhs;
		}
	}
}

impl<T: StackBlurrable, const N: usize> Mul<usize> for Argb<T, N> {
	type Output = Self;

	fn mul(self, rhs: usize) -> Self::Output {
		let mut iter = self.0.into_iter();
		Self([(); N].map(|_| iter.next().unwrap() * rhs))
	}
}

impl<T: StackBlurrable, const N: usize> Div<usize> for Argb<T, N> {
	type Output = Self;

	fn div(self, rhs: usize) -> Self::Output {
		let mut iter = self.0.into_iter();
		Self([(); N].map(|_| iter.next().unwrap() / rhs))
	}
}
