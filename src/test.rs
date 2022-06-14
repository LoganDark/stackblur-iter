extern crate test;

use test::Bencher;

#[bench]
fn blur_horiz(bencher: &mut Bencher) {
	const WIDTH: usize = 640;
	const HEIGHT: usize = 480;
	const RADIUS: usize = 128;

	let mut buf = vec![0; WIDTH * HEIGHT];

	bencher.iter(|| crate::blur_horiz(&mut buf, WIDTH, HEIGHT, RADIUS));
}
