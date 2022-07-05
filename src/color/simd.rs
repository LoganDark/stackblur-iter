use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};

pub use std::simd::{LaneCount, Simd, SupportedLaneCount};

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
pub struct StackBlurrableU32xN<const N: usize>(pub Simd<u32, N>) where LaneCount<N>: SupportedLaneCount;

impl<const N: usize> Add for StackBlurrableU32xN<N> where LaneCount<N>: SupportedLaneCount {
	type Output = Self;

	fn add(self, rhs: Self) -> Self::Output {
		Self(self.0 + rhs.0)
	}
}

impl<const N: usize> Sub for StackBlurrableU32xN<N> where LaneCount<N>: SupportedLaneCount {
	type Output = Self;

	fn sub(self, rhs: Self) -> Self::Output {
		Self(self.0 - rhs.0)
	}
}

impl<const N: usize> AddAssign for StackBlurrableU32xN<N> where LaneCount<N>: SupportedLaneCount {
	fn add_assign(&mut self, rhs: Self) {
		self.0 += rhs.0;
	}
}

impl<const N: usize> SubAssign for StackBlurrableU32xN<N> where LaneCount<N>: SupportedLaneCount {
	fn sub_assign(&mut self, rhs: Self) {
		self.0 -= rhs.0;
	}
}

impl<const N: usize> Mul<usize> for StackBlurrableU32xN<N> where LaneCount<N>: SupportedLaneCount {
	type Output = Self;

	fn mul(self, rhs: usize) -> Self::Output {
		Self(self.0 * Simd::<u32, N>::splat(rhs as u32))
	}
}

impl<const N: usize> Div<usize> for StackBlurrableU32xN<N> where LaneCount<N>: SupportedLaneCount {
	type Output = Self;

	fn div(self, rhs: usize) -> Self::Output {
		// This branch yields significant/10% speedups on my particular x86 CPU
		// I'm not sure why
		if N < 32 {
			// SIMD division
			Self(self.0 / Simd::<u32, N>::splat(rhs as u32))
		} else {
			// Individual division
			Self(Simd::<u32, N>::from_array(self.0.to_array().map(|e| (e as usize / rhs) as u32)))
		}
	}
}
