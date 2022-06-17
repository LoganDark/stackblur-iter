//! A fast, iterative, correct approach to Stackblur, resulting in a very smooth
//! and high-quality output, with no edge bleeding.
//!
//! This crate implements a tweaked version of the Stackblur algorithm requiring
//! `radius * 2 + 2` elements of space rather than `radius * 2 + 1`, which is a
//! small tradeoff for much-increased visual quality.
//!
//! The algorithm is exposed as an iterator ([`StackBlur`]) that can wrap any
//! other iterator that yields elements of [`StackBlurrable`]. The [`StackBlur`]
//! will then yield elements blurred by the specified radius.
//!
//! ## Benefits of this crate
//!
//! Stackblur is essentially constant-time. Regardless of the radius, it always
//! performs only 1 scan over the input iterator and outputs exactly the same
//! amount of elements.
//!
//! Additionally, it produces results that are comparable to slow and expensive
//! Gaussian blurs. As opposed to box blur which uses a basic rolling average,
//! Stackblur uses a weighted average where each output pixel is affected more
//! strongly by the inputs that were closest to it.
//!
//! Despite that, Stackblur does not perform much worse compared to naive box
//! blurs, and is quite cheap compared to full Gaussian blurs, at least for the
//! CPU. The implementation in this crate will most likely beat most unoptimized
//! blurs you can find on crates.io, as well as some optimized ones, and it is
//! extremely flexible and generic.
//!
//! For a full explanation of the improvements made to the Stackblur algorithm,
//! see the [`iter`] module.
//!
//! ## Comparison to the `stackblur` crate
//!
//! `stackblur` suffers from edge bleeding and flexibility problems. For
//! example, it can only operate on buffers of 32-bit integers, and expects them
//! to be packed linear ARGB pixels. Additionally, it cannot operate on a 2D
//! subslice of a buffer (like `imgref` allows for this crate), and it does not
//! offer any streaming iterators or documentation. And it also only supports
//! a blur radius of up to 255.
//!
//! ## Usage
//!
//! Aside from [`StackBlurrable`] and [`StackBlur`] which host their own
//! documentation, there are helper functions like [`blur`] and [`blur_argb`]
//! that can be used to interact with 2D image buffers, due to the fact that
//! doing so manually involves unsafe code (if you want no-copy).

#![cfg_attr(test, feature(test))]

use std::collections::VecDeque;

pub extern crate imgref;

use imgref::{Img, ImgRefMut};

#[cfg(test)]
mod test;

pub mod traits;
pub mod iter;
mod color;

use traits::StackBlurrable;
use iter::StackBlurIter;
use color::Argb;
use crate::iter::StackBlur;

struct RowsIter<T, B: StackBlurrable, F: FnMut(&T) -> B>(Img<*const [T]>, (usize, usize), F);

impl<T, B: StackBlurrable, F: FnMut(&T) -> B> Iterator for RowsIter<T, B, F> {
	type Item = B;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		if self.1.1 >= self.0.height() {
			None
		} else if self.1.0 >= self.0.width() {
			self.1.0 = 0;
			self.1.1 += 1;
			None
		} else {
			let elem = unsafe { (**self.0.buf()).get_unchecked(self.1.1 * self.0.stride() + self.1.0) };
			self.1.0 += 1;
			Some(self.2(elem))
		}
	}
}

struct RowsIterMut<T>(Img<*mut [T]>, (usize, usize));

impl<T> Iterator for RowsIterMut<T> {
	type Item = *mut T;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		if self.1.1 >= self.0.height() {
			None
		} else if self.1.0 >= self.0.width() {
			self.1.0 = 0;
			self.1.1 += 1;
			None
		} else {
			let ptr = unsafe { (**self.0.buf()).get_unchecked_mut(self.1.1 * self.0.stride() + self.1.0) as *mut T };
			self.1.0 += 1;
			Some(ptr)
		}
	}
}

struct ColsIter<T, B: StackBlurrable, F: FnMut(&T) -> B>(Img<*const [T]>, (usize, usize), F);

impl<T, B: StackBlurrable, F: FnMut(&T) -> B> Iterator for ColsIter<T, B, F> {
	type Item = B;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		if self.1.0 >= self.0.width() {
			None
		} else if self.1.1 >= self.0.height() {
			self.1.0 += 1;
			self.1.1 = 0;
			None
		} else {
			let elem = unsafe { (**self.0.buf()).get_unchecked(self.1.1 * self.0.stride() + self.1.0) };
			self.1.1 += 1;
			Some(self.2(elem))
		}
	}
}

struct ColsIterMut<T>(Img<*mut [T]>, (usize, usize));

impl<T> Iterator for ColsIterMut<T> {
	type Item = *mut T;

	#[inline]
	fn next(&mut self) -> Option<Self::Item> {
		if self.1.0 >= self.0.width() {
			None
		} else if self.1.1 >= self.0.height() {
			self.1.0 += 1;
			self.1.1 = 0;
			None
		} else {
			let ptr = unsafe { (**self.0.buf()).get_unchecked_mut(self.1.1 * self.0.stride() + self.1.0) as *mut T };
			self.1.1 += 1;
			Some(ptr)
		}
	}
}

/// Blurs a buffer on the X axis.
///
/// The provided closures are used to convert from the buffer's native pixel
/// format to [`StackBlurrable`] values that can be consumed by [`StackBlur`].
///
/// This is the generic version. If you have a common buffer format (packed
/// 32-bit integers), you can use [`blur_horiz_argb`] (linear RGB) or
/// [`blur_horiz_srgb`] (for sRGB).
pub fn blur_horiz<T, B: StackBlurrable>(
	buf: &mut ImgRefMut<T>,
	radius: usize,
	to_blurrable: impl FnMut(&T) -> B,
	mut to_pixel: impl FnMut(B) -> T
) {
	let buf_ptr = Img::new_stride(*buf.buf() as *const [T], buf.width(), buf.height(), buf.stride());

	let rows_iter = RowsIter(buf_ptr, (0, 0), to_blurrable);
	let mut blur = StackBlurIter::new(rows_iter, radius, VecDeque::new());

	for row in buf.rows_mut() {
		row.fill_with(|| to_pixel(blur.next().unwrap()));
		blur.next();
	}
}

/// Blurs a buffer on the Y axis.
///
/// The provided closures are used to convert from the buffer's native pixel
/// format to [`StackBlurrable`] values that can be consumed by [`StackBlur`].
///
/// This is the generic version. If you have a common buffer format (packed
/// 32-bit integers), you can use [`blur_vert_argb`] (linear RGB) or
/// [`blur_vert_srgb`] (for sRGB).
pub fn blur_vert<T, B: StackBlurrable>(
	buf: &mut ImgRefMut<T>,
	radius: usize,
	to_blurrable: impl FnMut(&T) -> B,
	mut to_pixel: impl FnMut(B) -> T
) {
	let buf_ptr = Img::new_stride(*buf.buf() as *const [T], buf.width(), buf.height(), buf.stride());
	let buf_ptr_mut = Img::new_stride(*buf.buf_mut() as *mut [T], buf.width(), buf.height(), buf.stride());

	let cols_iter = ColsIter(buf_ptr, (0, 0), to_blurrable);
	let mut blur = StackBlurIter::new(cols_iter, radius, VecDeque::new());
	let mut cols_iter = ColsIterMut(buf_ptr_mut, (0, 0));

	for _ in 0..buf.width() {
		for pixel in &mut cols_iter {
			unsafe { *pixel = to_pixel(blur.next().unwrap()) };
		}

		blur.next();
	}
}

/// Blurs a buffer on the X and Y axes.
///
/// The provided closures are used to convert from the buffer's native pixel
/// format to [`StackBlurrable`] values that can be consumed by [`StackBlur`].
///
/// This is the generic version. If you have a common buffer format (packed
/// 32-bit integers), you can use [`blur_argb`] (linear RGB) or [`blur_srgb`]
/// (for sRGB).
pub fn blur<T, B: StackBlurrable>(
	buf: &mut ImgRefMut<T>,
	radius: usize,
	to_blurrable: impl FnMut(&T) -> B,
	mut to_pixel: impl FnMut(B) -> T
) {
	let buf_ptr = Img::new_stride(*buf.buf() as *const [T], buf.width(), buf.height(), buf.stride());
	let buf_ptr_mut = Img::new_stride(*buf.buf_mut() as *mut [T], buf.width(), buf.height(), buf.stride());

	let rows_iter = RowsIter(buf_ptr, (0, 0), to_blurrable);
	let mut blur = StackBlurIter::new(rows_iter, radius, VecDeque::new());
	let mut rows_iter = RowsIterMut(buf_ptr_mut, (0, 0));
	let mut generators = vec![StackBlur::new(radius); buf.width()];

	for _ in 0..buf.height() {
		let mut preloaded = false;

		for (result, generator) in (&mut blur).zip(generators.iter_mut()) {
			if let Some(result) = generator.feed(Some(result)) {
				preloaded = true;
				unsafe { *rows_iter.next().unwrap() = to_pixel(result) };
			}
		}

		if preloaded {
			rows_iter.next();
		}
	}

	loop {
		let mut offloaded = false;

		for generator in generators.iter_mut() {
			if let Some(result) = generator.feed(None) {
				offloaded = true;
				unsafe { *rows_iter.next().unwrap() = to_pixel(result) };
			}
		}

		if offloaded {
			rows_iter.next();
		} else {
			break;
		}
	}
}

/// Blurs a buffer of 32-bit ARGB pixels on the X axis.
///
/// This is a version of [`blur_horiz`] with pre-filled conversion routines that
/// provide good results for blur radii <= 4096. Larger radii may overflow.
///
/// Note that this function is *linear*. For sRGB, see [`blur_horiz_srgb`].
pub fn blur_horiz_argb(buffer: &mut ImgRefMut<u32>, radius: usize) {
	blur_horiz(buffer, radius, |i| Argb::from_u32(*i), Argb::to_u32);
}

/// Blurs a buffer of 32-bit ARGB pixels on the Y axis.
///
/// This is a version of [`blur_vert`] with pre-filled conversion routines that
/// provide good results for blur radii <= 4096. Larger radii may overflow.
///
/// Note that this function is *linear*. For sRGB, see [`blur_vert_srgb`].
pub fn blur_vert_argb(buffer: &mut ImgRefMut<u32>, radius: usize) {
	blur_vert(buffer, radius, |i| Argb::from_u32(*i), Argb::to_u32);
}

/// Blurs a buffer of 32-bit ARGB pixels on both axes.
///
/// This is a version of [`blur`] with pre-filled conversion routines that
/// provide good results for blur radii <= 4096. Larger radii may overflow.
///
/// Note that this function is *linear*. For sRGB, see [`blur_srgb`].
pub fn blur_argb(buffer: &mut ImgRefMut<u32>, radius: usize) {
	blur(buffer, radius, |i| Argb::from_u32(*i), Argb::to_u32);
}

/// Blurs a buffer of 32-bit sRGB pixels on the X axis.
///
/// This is a version of [`blur_horiz`] with pre-filled conversion routines that
/// provide good results for blur radii <= 1536. Larger radii may overflow.
///
/// Note that this function uses *sRGB*. For linear, see [`blur_horiz_argb`].
#[cfg(any(doc, feature = "blend-srgb"))]
pub fn blur_horiz_srgb(buffer: &mut ImgRefMut<u32>, radius: usize) {
	blur_horiz(buffer, radius, |i| Argb::from_u32_srgb(*i), Argb::to_u32_srgb);
}

/// Blurs a buffer of 32-bit sRGB pixels on the Y axis.
///
/// This is a version of [`blur_vert`] with pre-filled conversion routines that
/// provide good results for blur radii <= 1536. Larger radii may overflow.
///
/// Note that this function uses *sRGB*. For linear, see [`blur_vert_argb`].
#[cfg(any(doc, feature = "blend-srgb"))]
pub fn blur_vert_srgb(buffer: &mut ImgRefMut<u32>, radius: usize) {
	blur_vert(buffer, radius, |i| Argb::from_u32_srgb(*i), Argb::to_u32_srgb);
}

/// Blurs a buffer of 32-bit sRGB pixels on both axes.
///
/// This is a version of [`blur`] with pre-filled conversion routines that
/// provide good results for blur radii <= 1024. Larger radii may overflow.
///
/// Note that this function uses *sRGB*. For linear, see [`blur_argb`].
#[cfg(any(doc, feature = "blend-srgb"))]
pub fn blur_srgb(buffer: &mut ImgRefMut<u32>, radius: usize) {
	blur(buffer, radius, |i| Argb::from_u32_srgb(*i), Argb::to_u32_srgb);
}
