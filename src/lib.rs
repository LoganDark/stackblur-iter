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

#[cfg(test)]
mod test;

pub mod traits;
pub mod iter;
mod color;

use traits::StackBlurrable;
use iter::StackBlur;
use color::Argb;

/// Blurs a buffer, assuming one element per pixel.
///
/// The provided closures are used to convert from the buffer's native pixel
/// format to [`StackBlurrable`] values that can be consumed by [`StackBlur`].
pub fn blur<T, B: StackBlurrable>(
	buffer: &mut ImgRefMut<T>,
	radius: usize,
	mut to_blurrable: impl FnMut(&T) -> B,
	mut to_pixel: impl FnMut(B) -> T
) {
	use imgref_iter::traits::{ImgIter, ImgIterMut, ImgIterPtrMut};

	let mut ops = VecDeque::new();

	let buffer_ptr = buffer.as_mut_ptr();
	let rows = unsafe { buffer_ptr.iter_rows_ptr_mut() }.zip(buffer.iter_rows());
	let cols = unsafe { buffer_ptr.iter_cols_ptr_mut() }.zip(buffer.iter_cols());

	for (write, read) in rows.chain(cols) {
		let mut blur = StackBlur::new(read.map(&mut to_blurrable), radius, ops);
		write.for_each(|place| unsafe { *place = to_pixel(blur.next().unwrap()) });
		ops = blur.into_ops();
	}
}

/// Blurs a buffer in parallel, assuming one element per pixel.
///
/// The provided closures are used to convert from the buffer's native pixel
/// format to [`StackBlurrable`] values that can be consumed by [`StackBlur`].
#[cfg(any(doc, feature = "rayon"))]
pub fn par_blur<T: Send + Sync, B: StackBlurrable + Send + Sync>(
	buffer: &mut ImgRefMut<T>,
	radius: usize,
	to_blurrable: impl Fn(&T) -> B + Sync,
	to_pixel: impl Fn(B) -> T + Sync
) {
	use imgref_iter::traits::{ImgIter, ImgIterMut, ImgIterPtrMut};
	#[cfg(not(doc))]
	use rayon::iter::{ParallelBridge, ParallelIterator};

	let mut opses = vec![Some(VecDeque::new()); rayon::current_num_threads()];
	let opses_ptr = unsafe { unique::Unique::new_unchecked(opses.as_mut_ptr()) };

	let buffer_ptr = buffer.as_mut_ptr();
	let rows = unsafe { buffer_ptr.iter_rows_ptr_mut() }.zip(buffer.iter_rows());
	let cols = unsafe { buffer_ptr.iter_cols_ptr_mut() }.zip(buffer.iter_cols());

	// This relies on an implementation detail (or bug) in `rayon` where chained
	// iterators that are then bridged will still execute in sequence.
	//
	// This may have to change in the future if that behavior is not guaranteed.
	rows.chain(cols).par_bridge().for_each(|(write, read)| {
		let ops_ref = unsafe { &mut *opses_ptr.as_ptr().add(rayon::current_thread_index().unwrap()) };
		let ops = ops_ref.take().unwrap();
		let mut blur = StackBlur::new(read.map(&to_blurrable), radius, ops);
		write.for_each(|place| unsafe { *place = to_pixel(blur.next().unwrap()) });
		ops_ref.replace(blur.into_ops());
	});
}

/// Blurs a buffer of 32-bit packed ARGB pixels (0xAARRGGBB).
///
/// This is a version of [`blur`] with pre-filled conversion routines that
/// provide good results for blur radii <= 4096. Larger radii may overflow.
///
/// Note that this function is *linear*. For sRGB, see [`blur_srgb`].
pub fn blur_argb(buffer: &mut ImgRefMut<u32>, radius: usize) {
	blur(buffer, radius, |i| Argb::from_u32(*i), Argb::to_u32);
}

/// Blurs a buffer of 32-bit packed sRGB pixels (0xAARRGGBB).
///
/// This is a version of [`blur`] with pre-filled conversion routines that
/// provide good results for blur radii <= 1536. Larger radii may overflow.
///
/// Note that this function uses *sRGB*. For linear, see [`blur_argb`].
#[cfg(any(doc, feature = "blend-srgb"))]
pub fn blur_srgb(buffer: &mut ImgRefMut<u32>, radius: usize) {
	blur(buffer, radius, |i| Argb::from_u32_srgb(*i), Argb::to_u32_srgb);
}

/// Blurs a buffer of 32-bit packed ARGB pixels (0xAARRGGBB) in parallel.
///
/// This is a version of [`par_blur`] with pre-filled conversion routines that
/// provide good results for blur radii <= 4096. Larger radii may overflow.
///
/// Note that this function is *linear*. For sRGB, see [`blur_srgb`].
#[cfg(any(doc, feature = "rayon"))]
pub fn par_blur_argb(buffer: &mut ImgRefMut<u32>, radius: usize) {
	par_blur(buffer, radius, |i| Argb::from_u32(*i), Argb::to_u32);
}

/// Blurs a buffer of 32-bit packed sRGB pixels (0xAARRGGBB) in parallel.
///
/// This is a version of [`par_blur`] with pre-filled conversion routines that
/// provide good results for blur radii <= 1536. Larger radii may overflow.
///
/// Note that this function uses *sRGB*. For linear, see [`blur_argb`].
#[cfg(any(doc, all(feature = "rayon", feature = "blend-srgb")))]
pub fn par_blur_srgb(buffer: &mut ImgRefMut<u32>, radius: usize) {
	par_blur(buffer, radius, |i| Argb::from_u32_srgb(*i), Argb::to_u32_srgb);
}
