//! This module implements the [`StackBlur`] struct and the improved version of
//! the Stackblur algorithm.
//!
//! ## The improved Stackblur algorithm
//!
//! As previously stated, this crate implements a modified version of Stackblur,
//! and understanding the original algorithm is key to understanding the
//! improvements that have been made to it.
//!
//! The original Stackblur is essentially a weighted box-blur. Instead of taking
//! the average of a flat array of pixels, like this:
//!
//! ```text
//! ( 00 + 01 + 02 + 03 + 04 + 05 + 06 ) / 7
//! ```
//!
//! it takes a weighted average of the pixels:
//!
//! ```text
//!              ( 03 +
//!           02 + 03 + 04 +
//!      01 + 02 + 03 + 04 + 05 +
//! 00 + 01 + 02 + 03 + 04 + 05 + 06 ) / 16
//! ```
//!
//! This is a rough approximation of a Gaussian blur, and in fact it's already
//! most of the way there to being a complete algorithm on its own. You can just
//! multiply each pixel by something like `radius + 1 - center_dist` when you
//! make your sum, then divide by `radius * (radius + 2) + 1`.
//!
//! But there are two problems with this:
//!
//! 1. That would be *O*(n(m * 2 + 1)²) complexity, where `n` is the number of
//!    pixels in the image, and `m` is the radius of the blur. This is basically
//!    just as expensive as running a proper convolution filter.
//!
//! 2. How do we handle pixels off the edge of the image?
//!
//! I'm scared of #1 so I'm going to handle #2 first. What most implementations
//! choose to do is just repeat the edge of the image:
//!
//! ```text
//!              ( 00 +
//!           00 + 00 + 01 +
//!      00 + 00 + 00 + 01 + 02 +
//! 00 + 00 + 00 + 00 + 01 + 02 + 03 ) / 16
//! ```
//!
//! But this creates even more problems, because the edge of the blur will be
//! quite biased towards the pixels that aren't even in the image. This is known
//! as "edge bleeding", where a single pixel at the edge of a blur can cause
//! very large and ugly artifacts to show up.
//!
//! The solution, of course, is to not calculate the denominator using that
//! equation from earlier, and instead incrementally update the denominator as
//! pixels are scanned, allowing you to sum a varying number of pixels:
//!
//! ```text
//! ( 00 +
//!   00 + 01 +
//!   00 + 01 + 02 +
//!   00 + 01 + 02 + 03 ) / 10
//!
//!      ( 01 +
//!   00 + 01 + 02 +
//!   00 + 01 + 02 + 03 +
//!   00 + 01 + 02 + 03 + 04 ) / 13
//!
//!           ( 02 +
//!        01 + 02 + 03 +
//!   00 + 01 + 02 + 03 + 04 +
//!   00 + 01 + 02 + 03 + 04 + 05 ) / 15
//!
//!                ( 03 +
//!             02 + 03 + 04 +
//!        01 + 02 + 03 + 04 + 05 +
//!   00 + 01 + 02 + 03 + 04 + 05 + 06 ) / 16
//! ```
//!
//! This is one of the improvements made to the Stackblur algorithm that is
//! implemented by this crate.
//!
//! Now for #1 - the complexity problem. It is possible to make a streaming
//! Stackblur that does not need to read any pixels out of order or more than
//! once, or even know how long the input is. In fact, that's exactly what this
//! crate does.
//!
//! First you fill up the cache with `radius + 1` pixels in order to be able to
//! make the first sum, then you start producing results until you run out of
//! pixels, then you produce the last `radius` results using what you have in
//! cache. I don't have the skill to explain the algorithm in full detail, but
//! it's open-source so you can look at it if you want.
//!
//! In this crate the "cache" is not actually the actual heap of values, as that
//! would be too slow. Instead, the "cache" is a list of changes to make to the
//! rate of change of the sum, and the denominator is updated incrementally in
//! response to the number of leading and trailing values changing.
//!
//! The reason the whole rate of change thing exists is so that adding to the
//! sum and registering it in the cache becomes *O*(1) instead of *O*(2n+1)
//! (where `n` is the radius). It's basically the most important thing that
//! makes the algorithm constant-time.

use std::collections::VecDeque;

use crate::traits::StackBlurrable;

/// An iterator that implements an improved Stackblur algorithm.
///
/// For any [`StackBlurrable`] element `T` and any iterator `I` over items of
/// type `T`, [`StackBlur`] will yield the same amount of elements as `I`,
/// blurred together according to the improved Stackblur algorithm as described
/// by the [`iter`][crate::iter] module documentation.
///
/// ## Usage
///
/// [`StackBlur`] just wraps another iterator with a radius and a cache (called
/// `ops` by [`StackBlur::new`]):
///
/// ```compile_fail
/// # use std::collections::VecDeque;
/// # use std::num::Wrapping;
/// # use stackblur_iter::iter::StackBlur;
/// #
/// let arr = [255u8, 0, 0, 0, 127, 0, 0, 0, 255u8];
/// let iter = arr.iter().copied().map(|i| Wrapping(i as usize));
/// let blur = StackBlur::new(iter, 2, VecDeque::new());
/// ```
///
/// That example unfortunately doesn't compile because `Wrapping<usize>` doesn't
/// implement `Mul<usize>` and `Div<usize>`, which are required for the
/// averaging that [`StackBlur`] performs. It's recommended to create a newtype
/// wrapper around whatever you plan on blurring, and implement all the traits
/// required by [`StackBlurrable`].
///
/// A [`StackBlur`] always yields exactly as many items as its inner iterator
/// does. Additionally, a non-fused iterator which repeats will cause the
/// [`StackBlur`] to repeat as well.
///
/// After using the [`StackBlur`], you can retrieve the [`VecDeque`] back out
/// of it by calling [`StackBlur::into_ops`].
pub struct StackBlur<T: StackBlurrable, I: Iterator<Item = T>> {
	iter: I,
	radius: usize,
	sum: T,
	rate: T,
	dnom: usize,
	ops: VecDeque<T>,
	leading: usize,
	trailing: usize,
	done: bool
}

impl<T: StackBlurrable, I: Iterator<Item = T>> StackBlur<T, I> {
	/// Creates a new [`StackBlur`] from the provided iterator, radius, and
	/// [`VecDeque`].
	///
	/// The iterator is not advanced until a call to [`StackBlur::next`].
	pub fn new(iter: I, radius: usize, ops: VecDeque<T>) -> Self {
		Self {
			iter,
			radius,
			sum: T::default(),
			rate: T::default(),
			dnom: 0,
			ops,
			leading: 0,
			trailing: 0,
			done: true
		}
	}

	/// Consumes this [`StackBlur`] and returns the inner [`VecDeque`].
	pub fn into_ops(self) -> VecDeque<T> {
		self.ops
	}

	fn init(&mut self) {
		self.done = false;

		let start = self.radius;
		let needed = start * 2 + 1;
		self.ops.reserve(needed.saturating_sub(self.ops.capacity()));
		self.ops.iter_mut().take(start).for_each(|place| *place = T::default());
		self.ops.resize_with(start, T::default);

		self.sum = T::default();
		self.rate = T::default();
		self.dnom = 0;
		self.leading = 0;
		self.trailing = 0;

		for sub in 0..=self.radius {
			let item = match self.iter.next() {
				Some(item) => item,
				None => break
			};

			let mul = self.radius + 1 - sub;
			self.sum += item.clone() * mul;
			self.rate += item.clone();
			self.dnom += mul;

			if self.dnom > mul {
				self.trailing += 1;
			}

			self.ops.push_back(item);
		}

		if self.dnom == 0 {
			self.done = true;
		}
	}
}

impl<T: StackBlurrable, I: Iterator<Item = T>> Iterator for StackBlur<T, I> {
	type Item = T;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		if self.done {
			self.init();

			if self.done {
				return None;
			}
		}

		let result = self.sum.clone() / self.dnom;

		self.rate -= self.ops[self.radius].clone() * 2;
		self.rate += self.ops.pop_front().unwrap();
		self.sum += self.rate.clone();

		if self.leading < self.radius {
			self.leading += 1;
			self.dnom += self.radius + 1 - self.leading;
		}

		// @formatter:off
		if self.radius > 0 && self.trailing == self.radius && let Some(item) = self.iter.next() {
			// @formatter:on
			self.sum += item.clone();
			self.rate += item.clone();
			self.ops.push_back(item);
		} else if self.trailing > 0 {
			self.dnom -= self.radius + 1 - self.trailing;
			self.trailing -= 1;
		} else if self.trailing == 0 {
			self.done = true;
		}

		Some(result)
	}
}
