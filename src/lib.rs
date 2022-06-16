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

#![feature(portable_simd, let_chains)]
#![cfg_attr(test, feature(test))]

use std::collections::VecDeque;

pub extern crate imgref;

use imgref::ImgRefMut;

#[cfg(test)]
mod test;

#[allow(unused_macros)]
macro_rules! coz {
	($name:ident $args:tt) => {
		#[cfg(feature = "coz")]
		coz::$name!$args;
	}
}

pub mod traits;
pub mod iter;
mod color;

use traits::StackBlurrable;
use iter::StackBlur;
use color::ARGB;

/// Blurs a buffer on the X axis.
///
/// The provided closures are used to convert from the buffer's native pixel
/// format to [`StackBlurrable`] values that can be consumed by [`StackBlur`].
///
/// This is the generic version. If you have a common buffer format (packed
/// 32-bit integers), you can use [`blur_horiz_argb`] (linear RGB) or
/// [`blur_horiz_srgb`] (for sRGB).
pub fn blur_horiz<T, B: StackBlurrable>(
	buffer: &mut ImgRefMut<T>,
	radius: usize,
	mut to_blurrable: impl FnMut(&T) -> B,
	mut to_pixel: impl FnMut(B) -> T
) {
	let mut ops = VecDeque::new();

	struct SlicePtrIter<T, B: StackBlurrable, F: FnMut(&T) -> B>(*const [T], F);

	impl<T, B: StackBlurrable, F: FnMut(&T) -> B> Iterator for SlicePtrIter<T, B, F> {
		type Item = B;

		#[inline]
		fn next(&mut self) -> Option<Self::Item> {
			if let Some((first, rest)) = unsafe { (*self.0).split_first() } {
				self.0 = rest as *const [T];
				Some(self.1(first))
			} else {
				None
			}
		}
	}

	for row in buffer.rows_mut() {
		let row = row as *mut [T];

		let iter = SlicePtrIter(row, &mut to_blurrable);
		let mut blur = StackBlur::new(iter, radius, ops);

		let mut ptr = row as *mut T;
		while let Some(pixel) = blur.next().map(&mut to_pixel) {
			// SAFETY: `blur` will always yield the same amount of items
			unsafe {
				*ptr = pixel;
				ptr = ptr.offset(1);
			};
		}

		ops = blur.into_ops();
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
	buffer: &mut ImgRefMut<T>,
	radius: usize,
	mut to_blurrable: impl FnMut(&T) -> B,
	mut to_pixel: impl FnMut(B) -> T
) {
	let mut ops = VecDeque::new();

	struct SlicePtrStrideIter<T, B: StackBlurrable, F: FnMut(&T) -> B>(*const [T], F, usize);

	impl<T, B: StackBlurrable, F: FnMut(&T) -> B> Iterator for SlicePtrStrideIter<T, B, F> {
		type Item = B;

		#[inline]
		fn next(&mut self) -> Option<Self::Item> {
			unsafe {
				let len = (*self.0).len();

				if len > 0 {
					let item = &*(self.0 as *mut T);
					self.0 = (*self.0).get_unchecked(std::cmp::min(len, self.2)..) as *const [T];
					Some(self.1(item))
				} else {
					None
				}
			}
		}
	}

	let buf_ptr = *buffer.buf() as *const [T];
	let buf_mut_ptr = *buffer.buf_mut() as *mut [T];
	let stride = buffer.stride();

	for col in 0..buffer.width() {
		let iter = SlicePtrStrideIter(unsafe { (*buf_ptr).get_unchecked(col..) as *const [T] }, &mut to_blurrable, stride);
		let mut blur = StackBlur::new(iter, radius, ops);

		let mut ptr = unsafe { (buf_mut_ptr as *mut T).offset(col as isize) };
		let mut rows_left = buffer.height();
		while let Some(pixel) = blur.next().map(&mut to_pixel) {
			// SAFETY: `blur` will always yield the same amount of items
			unsafe {
				*ptr = pixel;
				rows_left = rows_left - 1;

				if rows_left > 1 {
					ptr = ptr.offset(stride as isize);
				}
			};
		}

		ops = blur.into_ops();
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
	buffer: &mut ImgRefMut<T>,
	radius: usize,
	mut to_blurrable: impl FnMut(&T) -> B,
	mut to_pixel: impl FnMut(B) -> T
) {
	blur_horiz(buffer, radius, &mut to_blurrable, &mut to_pixel);
	blur_vert(buffer, radius, to_blurrable, to_pixel);
}

/// Blurs a buffer of 32-bit ARGB pixels on the X axis.
///
/// This is a version of [`blur_horiz`] with pre-filled conversion routines that
/// provide good results for blur radii <= 4096. Larger radii may overflow.
///
/// Note that this function is *linear*. For sRGB, see [`blur_horiz_srgb`].
pub fn blur_horiz_argb(buffer: &mut ImgRefMut<u32>, radius: usize) {
	blur_horiz(buffer, radius, |i| ARGB::from_u32(*i), ARGB::to_u32);
}

/// Blurs a buffer of 32-bit ARGB pixels on the Y axis.
///
/// This is a version of [`blur_vert`] with pre-filled conversion routines that
/// provide good results for blur radii <= 4096. Larger radii may overflow.
///
/// Note that this function is *linear*. For sRGB, see [`blur_vert_srgb`].
pub fn blur_vert_argb(buffer: &mut ImgRefMut<u32>, radius: usize) {
	blur_vert(buffer, radius, |i| ARGB::from_u32(*i), ARGB::to_u32);
}

/// Blurs a buffer of 32-bit ARGB pixels on both axes.
///
/// This is a version of [`blur`] with pre-filled conversion routines that
/// provide good results for blur radii <= 4096. Larger radii may overflow.
///
/// Note that this function is *linear*. For sRGB, see [`blur_srgb`].
pub fn blur_argb(buffer: &mut ImgRefMut<u32>, radius: usize) {
	blur_horiz_argb(buffer, radius);
	blur_vert_argb(buffer, radius);
}

/// Blurs a buffer of 32-bit sRGB pixels on the X axis.
///
/// This is a version of [`blur_horiz`] with pre-filled conversion routines that
/// provide good results for blur radii <= 1024. Larger radii may overflow.
///
/// Note that this function uses *sRGB*. For linear, see [`blur_horiz_argb`].
#[cfg(any(doc, feature = "blend-srgb"))]
pub fn blur_horiz_srgb(buffer: &mut ImgRefMut<u32>, radius: usize) {
	blur_horiz(buffer, radius, |i| ARGB::from_u32_srgb(*i), ARGB::to_u32_srgb);
}

/// Blurs a buffer of 32-bit sRGB pixels on the Y axis.
///
/// This is a version of [`blur_vert`] with pre-filled conversion routines that
/// provide good results for blur radii <= 1024. Larger radii may overflow.
///
/// Note that this function uses *sRGB*. For linear, see [`blur_vert_argb`].
#[cfg(any(doc, feature = "blend-srgb"))]
pub fn blur_vert_srgb(buffer: &mut ImgRefMut<u32>, radius: usize) {
	blur_vert(buffer, radius, |i| ARGB::from_u32_srgb(*i), ARGB::to_u32_srgb);
}

/// Blurs a buffer of 32-bit sRGB pixels on both axes.
///
/// This is a version of [`blur`] with pre-filled conversion routines that
/// provide good results for blur radii <= 1024. Larger radii may overflow.
///
/// Note that this function uses *sRGB*. For linear, see [`blur_argb`].
#[cfg(any(doc, feature = "blend-srgb"))]
pub fn blur_srgb(buffer: &mut ImgRefMut<u32>, radius: usize) {
	blur_horiz_srgb(buffer, radius);
	blur_vert_srgb(buffer, radius);
}
