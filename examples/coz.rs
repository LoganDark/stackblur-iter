fn main() {
	#[cfg(feature = "blend-srgb")] {
		use imgref::ImgVec;
		use stackblur_iter::{blur_horiz_srgb, blur_vert_srgb};

		coz::thread_init();

		const WIDTH: usize = 3940;
		const HEIGHT: usize = 2160;

		let mut buffer = ImgVec::new(vec![0u32; WIDTH * HEIGHT], WIDTH, HEIGHT);

		loop {
			coz::begin!("blur_horiz");
			blur_horiz_srgb(&mut buffer.as_mut(), 256);
			coz::end!("blur_horiz");

			coz::begin!("blur_vert");
			blur_vert_srgb(&mut buffer.as_mut(), 256);
			coz::end!("blur_vert");

			coz::progress!("blur");
		}
	}
}
