fn main() {
	#[cfg(all(feature = "coz", feature = "blend-srgb"))] {
		use imgref::ImgVec;
		use stackblur_iter::blur_srgb;

		coz::thread_init();

		const WIDTH: usize = 640;
		const HEIGHT: usize = 480;

		let mut buffer = ImgVec::new(vec![0u32; WIDTH * HEIGHT], WIDTH, HEIGHT);

		loop {
			coz::begin!("blur");
			blur_srgb(&mut buffer.as_mut(), 16);
			coz::end!("blur");
		}
	}
}
