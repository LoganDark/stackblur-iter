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

use imgref::ImgRefMut;
use imgref_iter::traits::{ImgIter, ImgIterPtrMut};

#[cfg(test)]
mod test;

pub mod traits;
pub mod iter;
mod color;

use traits::StackBlurrable;
use iter::StackBlur;
use color::Argb;

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

	for (write, read) in unsafe { buffer.iter_rows_ptr_mut() }.zip(buffer.iter_rows()) {
		let mut blur = StackBlur::new(read.map(&mut to_blurrable), radius, ops);
		write.for_each(|place| unsafe { *place = to_pixel(blur.next().unwrap()) });
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

	for (write, read) in unsafe { buffer.iter_cols_ptr_mut() }.zip(buffer.iter_cols()) {
		let mut blur = StackBlur::new(read.map(&mut to_blurrable), radius, ops);
		write.for_each(|place| unsafe { *place = to_pixel(blur.next().unwrap()) });
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
	let mut ops = VecDeque::new();

	for (write, read) in unsafe { buffer.iter_rows_ptr_mut() }.zip(buffer.iter_rows()) {
		let mut blur = StackBlur::new(read.map(&mut to_blurrable), radius, ops);
		write.for_each(|place| unsafe { *place = to_pixel(blur.next().unwrap()) });
		ops = blur.into_ops();
	}

	for (write, read) in unsafe { buffer.iter_cols_ptr_mut() }.zip(buffer.iter_cols()) {
		let mut blur = StackBlur::new(read.map(&mut to_blurrable), radius, ops);
		write.for_each(|place| unsafe { *place = to_pixel(blur.next().unwrap()) });
		ops = blur.into_ops();
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
