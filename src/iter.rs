//! This module implements the [`StackBlur`] generator, the [`StackBlurIter`]
//! iterator, and an improved version of the Stackblur algorithm.
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

#[derive(Clone, Eq, PartialEq, Debug)]
pub struct StackBlur<B: StackBlurrable> {
	radius: usize,
	sum: B,
	rate: B,
	dnom: usize,
	ops: VecDeque<B>,
	state: State
}

impl<B: StackBlurrable> StackBlur<B> {
	pub fn with_ops(radius: usize, ops: VecDeque<B>) -> Self {
		Self {
			radius,
			sum: B::default(),
			rate: B::default(),
			dnom: 0,
			ops,
			state: State::Preload { index: 0, trailing: 0 }
		}
	}

	pub fn new(radius: usize) -> Self {
		Self::with_ops(radius, VecDeque::new())
	}

	pub fn into_ops(self) -> VecDeque<B> {
		self.ops
	}
}

#[derive(Copy, Clone, Eq, PartialEq, Debug)]
enum State {
	Preload {
		index: usize,
		trailing: usize
	},

	Main {
		leading: usize,
		trailing: usize
	}
}

impl<B: StackBlurrable> StackBlur<B> {
	fn preload(&mut self, index: usize, trailing: usize, item: Option<B>) -> Option<B> {
		if let Some(item) = item {
			if index == 0 {
				let start = self.radius + 1;
				let needed = self.radius * 2 + 2;
				self.ops.reserve(needed.saturating_sub(self.ops.capacity()));
				self.ops.iter_mut().take(start).for_each(|place| *place = B::default());
				self.ops.resize_with(start, B::default);

				self.sum = B::default();
				self.rate = B::default();
				self.dnom = 0;
			} else if index > self.radius {
				self.state = State::Main { leading: 0, trailing };
				return Some(self.main(0, trailing, Some(item)));
			}

			let mul = self.radius + 1 - index;
			self.sum += item.clone() * mul;
			self.rate += item.clone();
			self.dnom += mul;

			self.ops.push_back(item);

			self.state = if index > self.radius {
				State::Main { leading: 0, trailing }
			} else if self.dnom > mul {
				State::Preload { index: index + 1, trailing: trailing + 1 }
			} else {
				State::Preload { index: index + 1, trailing }
			};

			None
		} else if index == 0 {
			None
		} else {
			self.state = State::Main { leading: 0, trailing };
			Some(self.main(0, trailing, None))
		}
	}

	fn main(&mut self, mut leading: usize, mut trailing: usize, item: Option<B>) -> B {
		let result = self.sum.clone() / self.dnom;

		self.rate += self.ops.pop_front().unwrap();
		self.rate -= self.ops[self.radius].clone() * 2;
		self.sum += self.rate.clone();

		if leading < self.radius {
			leading += 1;
			self.dnom += self.radius + 1 - leading;
		}

		if self.radius == 0 || trailing == self.radius {
			if let Some(item) = item {
				self.sum += item.clone();
				self.rate += item.clone();
				self.ops.push_back(item);
				self.state = State::Main { leading, trailing };
			} else if self.radius > 0 {
				self.dnom -= self.radius + 1 - trailing;
				trailing -= 1;
				self.state = State::Main { leading, trailing };
			} else {
				self.state = State::Preload { index: 0, trailing: 0 };
			}
		} else if trailing > 0 {
			assert!(item.is_none(), "fed item is not being consumed");
			self.dnom -= self.radius + 1 - trailing;
			trailing -= 1;
			self.state = State::Main { leading, trailing };
		} else if trailing == 0 {
			assert!(item.is_none(), "fed item is not being consumed");
			self.state = State::Preload { index: 0, trailing: 0 };
		}

		result
	}

	/// Feeds the generator one item. This may return `None` while the generator
	/// is warming up (up to the first `radius + 1` calls), but it will
	/// eventually start returning `Some`.
	///
	/// This method can panic if you feed it `Some` too soon after feeding it
	/// `None`. You must retrieve all items using `feed(None)` before you start
	/// feeding it `Some` again.
	#[inline]
	pub fn feed(&mut self, item: Option<B>) -> Option<B> {
		match self.state {
			State::Preload { index, trailing } => self.preload(index, trailing, item),
			State::Main { leading, trailing } => Some(self.main(leading, trailing, item))
		}
	}
}

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
#[derive(Clone, Eq, PartialEq, Debug)]
pub struct StackBlurIter<B: StackBlurrable, I: Iterator<Item = B>> {
	iter: I,
	generator: StackBlur<B>,
	resetting: bool
}

impl<B: StackBlurrable, I: Iterator<Item = B>> StackBlurIter<B, I> {
	/// Creates a new [`StackBlur`] from the provided iterator, radius, and
	/// [`VecDeque`].
	///
	/// The iterator is not advanced until a call to [`StackBlur::next`].
	#[inline]
	pub fn new(iter: I, radius: usize, ops: VecDeque<B>) -> Self {
		Self { iter, generator: StackBlur::with_ops(radius, ops), resetting: false }
	}

	/// Consumes this [`StackBlur`] and returns the inner [`VecDeque`].
	#[inline]
	pub fn into_ops(self) -> VecDeque<B> {
		self.generator.into_ops()
	}
}

impl<T: StackBlurrable, I: Iterator<Item = T>> Iterator for StackBlurIter<T, I> {
	type Item = T;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		if self.resetting {
			let result = self.generator.feed(None);
			self.resetting = result.is_some();
			result
		} else {
			for item in &mut self.iter {
				if let Some(result) = self.generator.feed(Some(item)) {
					return Some(result);
				}
			}

			let result = self.generator.feed(None);
			self.resetting = result.is_some();
			result
		}
	}
}
