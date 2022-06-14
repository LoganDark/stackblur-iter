#![feature(portable_simd)]
#![feature(test)]

use std::collections::VecDeque;
use std::iter::Peekable;
use std::ops::{Add, AddAssign, Div, Mul, Sub, SubAssign};
use std::simd::i32x4;

#[cfg(test)]
mod test;

pub trait StackBlurrable: Default + Clone + Add<Output = Self> + AddAssign + SubAssign + Mul<usize, Output = Self> + Div<usize, Output = Self> {}

impl<T: Default + Clone + Add<Output = T> + AddAssign + SubAssign + Mul<usize, Output = T> + Div<usize, Output = T>> StackBlurrable for T {}

pub struct StackBlur<T: StackBlurrable, I: Iterator<Item = T>> {
	iter: Peekable<I>,
	radius: usize,
	sum: T,
	dnom: usize,
	ops: VecDeque<T>,
	leading: usize,
	trailing: usize,
	done: bool
}

impl<T: StackBlurrable, I: Iterator<Item = T>> StackBlur<T, I> {
	pub fn new(iter: I, radius: usize, ops: VecDeque<T>) -> Self {
		Self { iter: iter.peekable(), radius, sum: T::default(), dnom: 0, ops, leading: 0, trailing: 0, done: true }
	}

	pub fn into_ops(self) -> VecDeque<T> {
		self.ops
	}

	fn init(&mut self) {
		self.done = false;

		self.ops.clear();
		self.ops.resize_with(self.radius * 2 + 1, T::default);

		self.sum = T::default();
		self.dnom = 0;
		self.leading = 0;
		self.trailing = 0;

		if self.iter.peek().is_none() {
			self.done = true;
			return;
		}

		for sub in 0..=self.radius {
			let item = match self.iter.next() {
				Some(item) => item,
				None => break
			};

			let mul = self.radius + 1 - sub;
			self.sum += item.clone() * mul;
			self.dnom += mul;

			if self.dnom > mul {
				self.trailing += 1;
			}

			for i in 0..=sub + self.radius {
				if i < sub {
					self.ops[i] += item.clone();
				} else {
					self.ops[i] -= item.clone();
				}
			}
		}
	}
}

impl<T: StackBlurrable, I: Iterator<Item = T>> Iterator for StackBlur<T, I> {
	type Item = T;

	fn next(&mut self) -> Option<Self::Item> {
		if self.done {
			self.init();

			if self.done {
				return None;
			}
		}

		let result = self.sum.clone() / self.dnom;

		self.sum += self.ops.pop_front().unwrap();
		self.ops.push_back(T::default());

		if self.leading < self.radius {
			self.leading += 1;
			self.dnom += self.radius + 1 - self.leading;
		}

		if self.trailing == self.radius && self.iter.peek().is_some() {
			let item = self.iter.next().unwrap();

			self.sum += item.clone();

			for i in 0..=self.radius * 2 {
				if i < self.radius {
					self.ops[i] += item.clone();
				} else {
					self.ops[i] -= item.clone();
				}
			}
		} else if self.trailing > 0 {
			self.dnom -= self.radius + 1 - self.trailing;
			self.trailing -= 1;
		} else if self.trailing == 0 {
			self.done = true;
		}

		Some(result)
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Debug, Default)]
struct ARGB(i32x4);

impl ARGB {
	fn from_argb(argb: u32) -> Self {
		let [a, r, g, b] = argb.to_be_bytes();
		Self(i32x4::from_array([a as i32, r as i32, g as i32, b as i32]))
	}

	fn to_argb(self) -> u32 {
		let [a, r, g, b] = self.0.to_array();
		u32::from_be_bytes([a as u8, r as u8, g as u8, b as u8])
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
		Self(self.0 / i32x4::splat(rhs as i32))
	}
}

pub fn blur_horiz(argb: &mut [u32], width: usize, height: usize, radius: usize) {
	debug_assert_eq!(argb.len(), width * height);

	let mut ops = VecDeque::new();

	for row in argb.chunks_exact_mut(width) {
		let not_safe = row as *mut [u32];

		let read = unsafe { (*not_safe).iter() }.copied().map(ARGB::from_argb);

		let mut iter = StackBlur::new(read, radius, ops);

		let mut index = 0usize;
		while let Some(argb) = iter.next() {
			unsafe { (*not_safe)[index] = argb.to_argb() };
			index += 1;
		}

		ops = iter.into_ops();
	}
}

pub fn blur_vert(argb: &mut [u32], width: usize, height: usize, radius: usize) {
	debug_assert_eq!(argb.len(), width * height);

	let mut ops = VecDeque::new();

	for col in 0..width {
		let not_safe = argb as *mut [u32];

		let read = unsafe { (*not_safe).iter() }.skip(col).step_by(width).copied().map(ARGB::from_argb);

		let mut iter = StackBlur::new(read, radius, ops);

		let mut row = 0usize;
		while let Some(argb) = iter.next() {
			unsafe { (*not_safe)[row * width + col] = argb.to_argb() };
			row += 1;
		}

		ops = iter.into_ops();
	}
}

pub fn blur(argb: &mut [u32], width: usize, height: usize, radius: usize) {
	blur_horiz(argb, width, height, radius);
	blur_vert(argb, width, height, radius);
}
