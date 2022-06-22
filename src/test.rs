extern crate test;

use std::num::{NonZeroU32, NonZeroUsize};
use test::Bencher;
use imgref::ImgVec;

const WIDTH: usize = 640;
const HEIGHT: usize = 480;

const WIDTH_NONZERO: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(WIDTH) };
const HEIGHT_NONZERO: NonZeroUsize = unsafe { NonZeroUsize::new_unchecked(HEIGHT) };

#[bench]
#[inline(never)]
fn blur_argb_16(bencher: &mut Bencher) {
	let mut buf = ImgVec::new(vec![0; WIDTH * HEIGHT], WIDTH, HEIGHT);
	bencher.iter(|| crate::blur_argb(&mut buf.as_mut(), 16));
}

#[bench]
#[inline(never)]
fn blur_argb_128(bencher: &mut Bencher) {
	let mut buf = ImgVec::new(vec![0; WIDTH * HEIGHT], WIDTH, HEIGHT);
	bencher.iter(|| crate::blur_argb(&mut buf.as_mut(), 128));
}

#[bench]
#[inline(never)]
fn blur_argb_1024(bencher: &mut Bencher) {
	let mut buf = ImgVec::new(vec![0; WIDTH * HEIGHT], WIDTH, HEIGHT);
	bencher.iter(|| crate::blur_argb(&mut buf.as_mut(), 1024));
}

#[cfg(feature = "blend-srgb")]
#[bench]
#[inline(never)]
fn par_blur_srgb_16(bencher: &mut Bencher) {
	let mut buf = ImgVec::new(vec![0; WIDTH * HEIGHT], WIDTH, HEIGHT);
	bencher.iter(|| crate::par_blur_srgb(&mut buf.as_mut(), 16));
}

#[cfg(all(feature = "blend-srgb", feature = "rayon"))]
#[bench]
#[inline(never)]
fn par_blur_srgb_128(bencher: &mut Bencher) {
	let mut buf = ImgVec::new(vec![0; WIDTH * HEIGHT], WIDTH, HEIGHT);
	bencher.iter(|| crate::par_blur_srgb(&mut buf.as_mut(), 128));
}

#[cfg(all(feature = "blend-srgb", feature = "rayon"))]
#[bench]
#[inline(never)]
fn par_blur_srgb_1024(bencher: &mut Bencher) {
	let mut buf = ImgVec::new(vec![0; WIDTH * HEIGHT], WIDTH, HEIGHT);
	bencher.iter(|| crate::par_blur_srgb(&mut buf.as_mut(), 1024));
}

#[bench]
#[inline(never)]
fn stackblur_16_horiz(bencher: &mut Bencher) {
	let mut buf = vec![0; WIDTH * HEIGHT];
	bencher.iter(|| stackblur::blur_horiz(&mut buf, WIDTH_NONZERO, unsafe { NonZeroU32::new_unchecked(16) }));
}

#[bench]
#[inline(never)]
fn stackblur_128_horiz(bencher: &mut Bencher) {
	let mut buf = vec![0; WIDTH * HEIGHT];
	bencher.iter(|| stackblur::blur_horiz(&mut buf, WIDTH_NONZERO, unsafe { NonZeroU32::new_unchecked(128) }));
}

#[bench]
#[inline(never)]
fn stackblur_1024_horiz(bencher: &mut Bencher) {
	let mut buf = vec![0; WIDTH * HEIGHT];
	bencher.iter(|| stackblur::blur_horiz(&mut buf, WIDTH_NONZERO, unsafe { NonZeroU32::new_unchecked(1024) }));
}

#[bench]
#[inline(never)]
fn stackblur_16_vert(bencher: &mut Bencher) {
	let mut buf = vec![0; WIDTH * HEIGHT];
	bencher.iter(|| stackblur::blur_vert(&mut buf, WIDTH_NONZERO, HEIGHT_NONZERO, unsafe { NonZeroU32::new_unchecked(16) }));
}

#[bench]
#[inline(never)]
fn stackblur_128_vert(bencher: &mut Bencher) {
	let mut buf = vec![0; WIDTH * HEIGHT];
	bencher.iter(|| stackblur::blur_vert(&mut buf, WIDTH_NONZERO, HEIGHT_NONZERO, unsafe { NonZeroU32::new_unchecked(128) }));
}

#[bench]
#[inline(never)]
fn stackblur_1024_vert(bencher: &mut Bencher) {
	let mut buf = vec![0; WIDTH * HEIGHT];
	bencher.iter(|| stackblur::blur_vert(&mut buf, WIDTH_NONZERO, HEIGHT_NONZERO, unsafe { NonZeroU32::new_unchecked(1024) }));
}

#[bench]
#[inline(never)]
fn stackblur_16(bencher: &mut Bencher) {
	let mut buf = vec![0; WIDTH * HEIGHT];
	bencher.iter(|| stackblur::blur(&mut buf, WIDTH_NONZERO, HEIGHT_NONZERO, unsafe { NonZeroU32::new_unchecked(16) }));
}

#[bench]
#[inline(never)]
fn stackblur_128(bencher: &mut Bencher) {
	let mut buf = vec![0; WIDTH * HEIGHT];
	bencher.iter(|| stackblur::blur(&mut buf, WIDTH_NONZERO, HEIGHT_NONZERO, unsafe { NonZeroU32::new_unchecked(128) }));
}

#[bench]
#[inline(never)]
fn stackblur_1024(bencher: &mut Bencher) {
	let mut buf = vec![0; WIDTH * HEIGHT];
	bencher.iter(|| stackblur::blur(&mut buf, WIDTH_NONZERO, HEIGHT_NONZERO, unsafe { NonZeroU32::new_unchecked(1024) }));
}

#[cfg(all(feature = "blend-srgb", feature = "simd"))]
#[bench]
#[inline(never)]
fn simd_speed_01(bencher: &mut Bencher) {
	let mut buf = ImgVec::new(vec![0; WIDTH * HEIGHT], WIDTH, HEIGHT);
	bencher.iter(|| crate::blur_srgb_simd::<1>(&mut buf.as_mut(), 16));
}

#[cfg(all(feature = "blend-srgb", feature = "simd"))]
#[bench]
#[inline(never)]
fn simd_speed_02(bencher: &mut Bencher) {
	let mut buf = ImgVec::new(vec![0; WIDTH * HEIGHT], WIDTH, HEIGHT);
	bencher.iter(|| crate::blur_srgb_simd::<2>(&mut buf.as_mut(), 16));
}

#[cfg(all(feature = "blend-srgb", feature = "simd"))]
#[bench]
#[inline(never)]
fn simd_speed_04(bencher: &mut Bencher) {
	let mut buf = ImgVec::new(vec![0; WIDTH * HEIGHT], WIDTH, HEIGHT);
	bencher.iter(|| crate::blur_srgb_simd::<4>(&mut buf.as_mut(), 16));
}

#[cfg(all(feature = "blend-srgb", feature = "simd"))]
#[bench]
#[inline(never)]
fn simd_speed_08(bencher: &mut Bencher) {
	let mut buf = ImgVec::new(vec![0; WIDTH * HEIGHT], WIDTH, HEIGHT);
	bencher.iter(|| crate::blur_srgb_simd::<8>(&mut buf.as_mut(), 16));
}

#[cfg(all(feature = "blend-srgb", feature = "simd"))]
#[bench]
#[inline(never)]
fn simd_speed_16(bencher: &mut Bencher) {
	let mut buf = ImgVec::new(vec![0; WIDTH * HEIGHT], WIDTH, HEIGHT);
	bencher.iter(|| crate::blur_srgb_simd::<16>(&mut buf.as_mut(), 16));
}

#[cfg(all(feature = "blend-srgb", feature = "simd"))]
#[bench]
#[inline(never)]
fn simd_speed_32(bencher: &mut Bencher) {
	let mut buf = ImgVec::new(vec![0; WIDTH * HEIGHT], WIDTH, HEIGHT);
	bencher.iter(|| crate::blur_srgb_simd::<32>(&mut buf.as_mut(), 16));
}

#[cfg(all(feature = "blend-srgb", feature = "simd"))]
#[bench]
#[inline(never)]
fn simd_speed_64(bencher: &mut Bencher) {
	let mut buf = ImgVec::new(vec![0; WIDTH * HEIGHT], WIDTH, HEIGHT);
	bencher.iter(|| crate::blur_srgb_simd::<64>(&mut buf.as_mut(), 16));
}
