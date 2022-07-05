use imgref::{Img, ImgRefMut};

const WIDTH: usize = 640;
const HEIGHT: usize = 480;

static mut BUFFER: [u32; WIDTH * HEIGHT] = [0; WIDTH * HEIGHT];

#[inline(always)]
fn img() -> ImgRefMut<'static, u32> {
	Img::new(unsafe { &mut BUFFER[..] }, WIDTH, HEIGHT)
}

use stackblur_iter::{simd_blur_argb, simd_blur_srgb};

fn blur_argb_u32x1() { simd_blur_argb::<1>(&mut img(), 16) }
fn blur_argb_u32x2() { simd_blur_argb::<2>(&mut img(), 16) }
fn blur_argb_u32x4() { simd_blur_argb::<4>(&mut img(), 16) }
fn blur_argb_u32x8() { simd_blur_argb::<8>(&mut img(), 16) }
fn blur_argb_u32x16() { simd_blur_argb::<16>(&mut img(), 16) }
fn blur_argb_u32x32() { simd_blur_argb::<32>(&mut img(), 16) }
fn blur_argb_u32x64() { simd_blur_argb::<64>(&mut img(), 16) }

fn blur_srgb_u32x1() { simd_blur_srgb::<1>(&mut img(), 16) }
fn blur_srgb_u32x2() { simd_blur_srgb::<2>(&mut img(), 16) }
fn blur_srgb_u32x4() { simd_blur_srgb::<4>(&mut img(), 16) }
fn blur_srgb_u32x8() { simd_blur_srgb::<8>(&mut img(), 16) }
fn blur_srgb_u32x16() { simd_blur_srgb::<16>(&mut img(), 16) }
fn blur_srgb_u32x32() { simd_blur_srgb::<32>(&mut img(), 16) }
fn blur_srgb_u32x64() { simd_blur_srgb::<64>(&mut img(), 16) }

iai::main!(
	blur_argb_u32x1,
	blur_argb_u32x2,
	blur_argb_u32x4,
	blur_argb_u32x8,
	blur_argb_u32x16,
	blur_argb_u32x32,
	blur_argb_u32x64,

	blur_srgb_u32x1,
	blur_srgb_u32x2,
	blur_srgb_u32x4,
	blur_srgb_u32x8,
	blur_srgb_u32x16,
	blur_srgb_u32x32,
	blur_srgb_u32x64
);
