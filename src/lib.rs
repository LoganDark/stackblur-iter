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

#![cfg_attr(feature = "simd", feature(portable_simd))]
#![cfg_attr(test, feature(test))]

use std::collections::VecDeque;
#[cfg(any(doc, feature = "simd"))]
use std::simd::{LaneCount, SupportedLaneCount};

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

	for iter in [rows, cols].into_iter() {
		iter.par_bridge().for_each(|(write, read)| {
			let ops_ref = unsafe { &mut *opses_ptr.as_ptr().add(rayon::current_thread_index().unwrap()) };
			let ops = ops_ref.take().unwrap();
			let mut blur = StackBlur::new(read.map(&to_blurrable), radius, ops);
			write.for_each(|place| unsafe { *place = to_pixel(blur.next().unwrap()) });
			ops_ref.replace(blur.into_ops());
		});
	}
}


/// Blurs a buffer with SIMD, assuming one element per pixel.
///
/// The provided closures are used to convert from the buffer's native pixel
/// format to [`StackBlurrable`] values that can be consumed by [`StackBlur`].
#[cfg(any(doc, feature = "simd"))]
pub fn simd_blur<T, Bsimd: StackBlurrable, Bsingle: StackBlurrable, const LANES: usize>(
	buffer: &mut ImgRefMut<T>,
	radius: usize,
	mut to_blurrable_simd: impl FnMut([&T; LANES]) -> Bsimd,
	mut to_pixel_simd: impl FnMut(Bsimd) -> [T; LANES],
	mut to_blurrable_single: impl FnMut(&T) -> Bsingle,
	mut to_pixel_single: impl FnMut(Bsingle) -> T
) where LaneCount<LANES>: SupportedLaneCount {
	#[cfg(not(doc))]
	use imgref_iter::traits::{ImgIterMut, ImgSimdIter, ImgSimdIterPtrMut};
	#[cfg(not(doc))]
	use imgref_iter::iter::{SimdIterWindow, SimdIterWindowPtrMut};

	let mut ops_simd = VecDeque::new();
	let mut ops_single = VecDeque::new();

	let buffer_ptr = buffer.as_mut_ptr();
	let rows = unsafe { buffer_ptr.simd_iter_rows_ptr_mut::<LANES>() }.zip(buffer.simd_iter_rows::<LANES>());
	let cols = unsafe { buffer_ptr.simd_iter_cols_ptr_mut::<LANES>() }.zip(buffer.simd_iter_cols::<LANES>());

	for (write, read) in rows.chain(cols) {
		match (write, read) {
			(SimdIterWindowPtrMut::Simd(write), SimdIterWindow::Simd(read)) => {
				let mut blur = StackBlur::new(read.map(&mut to_blurrable_simd), radius, ops_simd);
				write.for_each(|place| place.into_iter().zip(to_pixel_simd(blur.next().unwrap())).for_each(|(place, pixel)| unsafe { *place = pixel }));
				ops_simd = blur.into_ops();
			}

			(SimdIterWindowPtrMut::Single(write), SimdIterWindow::Single(read)) => {
				let mut blur = StackBlur::new(read.map(&mut to_blurrable_single), radius, ops_single);
				write.for_each(|place| unsafe { *place = to_pixel_single(blur.next().unwrap()) });
				ops_single = blur.into_ops();
			}

			_ => unreachable!()
		}
	}
}

/// Blurs a buffer with SIMD in parallel, assuming one element per pixel.
///
/// The provided closures are used to convert from the buffer's native pixel
/// format to [`StackBlurrable`] values that can be consumed by [`StackBlur`].
#[cfg(any(doc, all(feature = "rayon", feature = "simd")))]
pub fn par_simd_blur<T: Send + Sync, Bsimd: StackBlurrable + Send + Sync, Bsingle: StackBlurrable + Send + Sync, const LANES: usize>(
	buffer: &mut ImgRefMut<T>,
	radius: usize,
	to_blurrable_simd: impl Fn([&T; LANES]) -> Bsimd + Sync,
	to_pixel_simd: impl Fn(Bsimd) -> [T; LANES] + Sync,
	to_blurrable_single: impl Fn(&T) -> Bsingle + Sync,
	to_pixel_single: impl Fn(Bsingle) -> T + Sync
) where LaneCount<LANES>: SupportedLaneCount {
	#[cfg(not(doc))]
	use imgref_iter::traits::{ImgIterMut, ImgSimdIter, ImgSimdIterPtrMut};
	#[cfg(not(doc))]
	use rayon::iter::{ParallelBridge, ParallelIterator};
	#[cfg(not(doc))]
	use imgref_iter::iter::{SimdIterWindow, SimdIterWindowPtrMut};

	let mut opses_simd = vec![Some(VecDeque::new()); rayon::current_num_threads()];
	let opses_simd_ptr = unsafe { unique::Unique::new_unchecked(opses_simd.as_mut_ptr()) };

	let mut opses_single = vec![Some(VecDeque::new()); rayon::current_num_threads()];
	let opses_single_ptr = unsafe { unique::Unique::new_unchecked(opses_single.as_mut_ptr()) };

	let buffer_ptr = buffer.as_mut_ptr();
	let rows = unsafe { buffer_ptr.simd_iter_rows_ptr_mut::<LANES>() }.zip(buffer.simd_iter_rows::<LANES>());
	let cols = unsafe { buffer_ptr.simd_iter_cols_ptr_mut::<LANES>() }.zip(buffer.simd_iter_cols::<LANES>());

	for iter in [rows, cols].into_iter() {
		iter.par_bridge().for_each(|(write, read)| match (write, read) {
			(SimdIterWindowPtrMut::Simd(write), SimdIterWindow::Simd(read)) => {
				let ops_ref = unsafe { &mut *opses_simd_ptr.as_ptr().add(rayon::current_thread_index().unwrap()) };
				let ops = ops_ref.take().unwrap();
				let mut blur = StackBlur::new(read.map(&to_blurrable_simd), radius, ops);
				write.for_each(|place| place.into_iter().zip(to_pixel_simd(blur.next().unwrap())).for_each(|(place, pixel)| unsafe { *place = pixel }));
				ops_ref.replace(blur.into_ops());
			}

			(SimdIterWindowPtrMut::Single(write), SimdIterWindow::Single(read)) => {
				let ops_ref = unsafe { &mut *opses_single_ptr.as_ptr().add(rayon::current_thread_index().unwrap()) };
				let ops = ops_ref.take().unwrap();
				let mut blur = StackBlur::new(read.map(&to_blurrable_single), radius, ops);
				write.for_each(|place| unsafe { *place = to_pixel_single(blur.next().unwrap()) });
				ops_ref.replace(blur.into_ops());
			}

			_ => unreachable!()
		});
	}
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
/// Note that this function is *linear*. For sRGB, see [`par_blur_srgb`].
#[cfg(any(doc, feature = "rayon"))]
pub fn par_blur_argb(buffer: &mut ImgRefMut<u32>, radius: usize) {
	par_blur(buffer, radius, |i| Argb::from_u32(*i), Argb::to_u32);
}

/// Blurs a buffer of 32-bit packed sRGB pixels (0xAARRGGBB) in parallel.
///
/// This is a version of [`par_blur`] with pre-filled conversion routines that
/// provide good results for blur radii <= 1536. Larger radii may overflow.
///
/// Note that this function uses *sRGB*. For linear, see [`par_blur_argb`].
#[cfg(any(doc, all(feature = "rayon", feature = "blend-srgb")))]
pub fn par_blur_srgb(buffer: &mut ImgRefMut<u32>, radius: usize) {
	par_blur(buffer, radius, |i| Argb::from_u32_srgb(*i), Argb::to_u32_srgb);
}

/// Blurs a buffer of 32-bit packed ARGB pixels (0xAARRGGBB) with SIMD.
///
/// This is a version of [`simd_blur`] with pre-filled conversion routines that
/// provide good results for blur radii <= 4096. Larger radii may overflow.
///
/// Note that this function is *linear*. For sRGB, see [`simd_blur_srgb`].
#[cfg(any(doc, feature = "simd"))]
pub fn simd_blur_argb<const LANES: usize>(buffer: &mut ImgRefMut<u32>, radius: usize) where LaneCount<LANES>: SupportedLaneCount {
	simd_blur(buffer, radius,
		|i: [&u32; LANES]| Argb::from_u32xN(i.map(u32::clone)), Argb::to_u32xN,
		|i| Argb::from_u32(*i), Argb::to_u32
	);
}

/// Blurs a buffer of 32-bit packed sRGB pixels (0xAARRGGBB) with SIMD.
///
/// This is a version of [`simd_blur`] with pre-filled conversion routines that
/// provide good results for blur radii <= 1536. Larger radii may overflow.
///
/// Note that this function uses *sRGB*. For linear, see [`simd_blur_argb`].
#[cfg(any(doc, all(feature = "simd", feature = "blend-srgb")))]
pub fn simd_blur_srgb<const LANES: usize>(buffer: &mut ImgRefMut<u32>, radius: usize) where LaneCount<LANES>: SupportedLaneCount {
	simd_blur(buffer, radius,
		|i: [&u32; LANES]| Argb::from_u32xN_srgb(i.map(u32::clone)), Argb::to_u32xN_srgb,
		|i| Argb::from_u32_srgb(*i), Argb::to_u32_srgb
	);
}

/// Blurs a buffer of 32-bit packed ARGB pixels (0xAARRGGBB) with SIMD in
/// parallel.
///
/// This is a version of [`par_simd_blur`] with pre-filled conversion routines
/// that provide good results for blur radii <= 4096. Larger radii may overflow.
///
/// Note that this function is *linear*. For sRGB, see [`par_simd_blur_srgb`].
#[cfg(any(doc, all(feature = "rayon", feature = "simd")))]
pub fn par_simd_blur_argb<const LANES: usize>(buffer: &mut ImgRefMut<u32>, radius: usize) where LaneCount<LANES>: SupportedLaneCount {
	par_simd_blur(buffer, radius,
		|i: [&u32; LANES]| Argb::from_u32xN(i.map(u32::clone)), Argb::to_u32xN,
		|i| Argb::from_u32(*i), Argb::to_u32
	);
}

/// Blurs a buffer of 32-bit packed sRGB pixels (0xAARRGGBB) with SIMD in
/// parallel.
///
/// This is a version of [`par_simd_blur`] with pre-filled conversion routines
/// that provide good results for blur radii <= 1536. Larger radii may overflow.
///
/// Note that this function uses *sRGB*. For linear, see [`par_simd_blur_argb`].
#[cfg(any(doc, all(feature = "rayon", feature = "simd", feature = "blend-srgb")))]
pub fn par_simd_blur_srgb<const LANES: usize>(buffer: &mut ImgRefMut<u32>, radius: usize) where LaneCount<LANES>: SupportedLaneCount {
	par_simd_blur(buffer, radius,
		|i: [&u32; LANES]| Argb::from_u32xN_srgb(i.map(u32::clone)), Argb::to_u32xN_srgb,
		|i| Argb::from_u32_srgb(*i), Argb::to_u32_srgb
	);
}
