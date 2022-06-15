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
fn blur_16(bencher: &mut Bencher) {
	let mut buf = ImgVec::new(vec![0; WIDTH * HEIGHT], WIDTH, HEIGHT);
	bencher.iter(|| crate::blur_argb(&mut buf.as_mut(), 16));
}

#[bench]
#[inline(never)]
fn blur_128(bencher: &mut Bencher) {
	let mut buf = ImgVec::new(vec![0; WIDTH * HEIGHT], WIDTH, HEIGHT);
	bencher.iter(|| crate::blur_argb(&mut buf.as_mut(), 128));
}

#[bench]
#[inline(never)]
fn blur_1024(bencher: &mut Bencher) {
	let mut buf = ImgVec::new(vec![0; WIDTH * HEIGHT], WIDTH, HEIGHT);
	bencher.iter(|| crate::blur_argb(&mut buf.as_mut(), 1024));
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
