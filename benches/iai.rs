use imgref::{Img, ImgRefMut};

const WIDTH: usize = 640;
const HEIGHT: usize = 480;

static mut BUFFER: [u32; WIDTH * HEIGHT] = [0; WIDTH * HEIGHT];

#[inline(always)]
fn img() -> ImgRefMut<'static, u32> {
	Img::new(unsafe { &mut BUFFER[..] }, WIDTH, HEIGHT)
}

fn blur_horiz_argb_16() { stackblur_iter::blur_horiz_argb(&mut img(), 16) }
fn blur_vert_argb_16() { stackblur_iter::blur_vert_argb(&mut img(), 16) }
fn blur_argb_16() { stackblur_iter::blur_argb(&mut img(), 16) }

fn blur_horiz_argb_128() { stackblur_iter::blur_horiz_argb(&mut img(), 128) }
fn blur_vert_argb_128() { stackblur_iter::blur_vert_argb(&mut img(), 128) }
fn blur_argb_128() { stackblur_iter::blur_argb(&mut img(), 128) }

fn blur_horiz_argb_1024() { stackblur_iter::blur_horiz_argb(&mut img(), 1024) }
fn blur_vert_argb_1024() { stackblur_iter::blur_vert_argb(&mut img(), 1024) }
fn blur_argb_1024() { stackblur_iter::blur_argb(&mut img(), 1024) }

#[cfg(feature = "blend-srgb")]
fn blur_horiz_srgb_16() { stackblur_iter::blur_horiz_srgb(&mut img(), 16) }
#[cfg(feature = "blend-srgb")]
fn blur_vert_srgb_16() { stackblur_iter::blur_vert_srgb(&mut img(), 16) }
#[cfg(feature = "blend-srgb")]
fn blur_srgb_16() { stackblur_iter::blur_srgb(&mut img(), 16) }

#[cfg(feature = "blend-srgb")]
fn blur_horiz_srgb_128() { stackblur_iter::blur_horiz_srgb(&mut img(), 128) }
#[cfg(feature = "blend-srgb")]
fn blur_vert_srgb_128() { stackblur_iter::blur_vert_srgb(&mut img(), 128) }
#[cfg(feature = "blend-srgb")]
fn blur_srgb_128() { stackblur_iter::blur_srgb(&mut img(), 128) }

#[cfg(feature = "blend-srgb")]
fn blur_horiz_srgb_1024() { stackblur_iter::blur_horiz_srgb(&mut img(), 1024) }
#[cfg(feature = "blend-srgb")]
fn blur_vert_srgb_1024() { stackblur_iter::blur_vert_srgb(&mut img(), 1024) }
#[cfg(feature = "blend-srgb")]
fn blur_srgb_1024() { stackblur_iter::blur_srgb(&mut img(), 1024) }

#[cfg(not(feature = "blend-srgb"))]
iai::main!(
	blur_horiz_argb_16,
	blur_vert_argb_16,
	blur_argb_16,

	blur_horiz_argb_128,
	blur_vert_argb_128,
	blur_argb_128,

	blur_horiz_argb_1024,
	blur_vert_argb_1024,
	blur_argb_1024
);

#[cfg(feature = "blend-srgb")]
iai::main!(
	blur_horiz_argb_16,
	blur_vert_argb_16,
	blur_argb_16,

	blur_horiz_argb_128,
	blur_vert_argb_128,
	blur_argb_128,

	blur_horiz_argb_1024,
	blur_vert_argb_1024,
	blur_argb_1024,

	blur_horiz_srgb_16,
	blur_vert_srgb_16,
	blur_srgb_16,

	blur_horiz_srgb_128,
	blur_vert_srgb_128,
	blur_srgb_128,

	blur_horiz_srgb_1024,
	blur_vert_srgb_1024,
	blur_srgb_1024
);
