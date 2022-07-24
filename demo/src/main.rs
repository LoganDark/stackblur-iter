use std::num::{NonZeroU32, NonZeroUsize};
use std::time::Instant;
use rayon::iter::{IndexedParallelIterator, IntoParallelRefMutIterator, ParallelIterator};
use softbuffer::GraphicsContext;
use stackblur_iter::*;
use stackblur_iter::imgref::ImgRefMut;
use winit::dpi::PhysicalSize;
use winit::event::{ElementState, Event, KeyboardInput, VirtualKeyCode, WindowEvent};
use winit::event_loop::{ControlFlow, EventLoop};
use winit::platform::run_return::EventLoopExtRunReturn;
use winit::window::{Window, WindowBuilder};

fn main() {
	const OLD_STACKBLUR_TITLE: &str = "old stackblur demo (press B to toggle)";
	const STACKBLUR_ITER_TITLE: &str = "stackblur-iter demo (press B to toggle)";

	let mut target = EventLoop::new();

	let window = WindowBuilder::new()
		//.with_transparent(true)
		.with_title(STACKBLUR_ITER_TITLE)
		.with_visible(false)
		.build(&target)
		.expect("couldn't open window");

	let id = window.id();

	let mut ctx = unsafe { GraphicsContext::new(window) }
		.expect("couldn't open graphics context");

	let mut buffer = vec![];

	let mut first = true;

	let mut use_old_stackblur = false;

	fn redraw(ctx: &mut GraphicsContext<Window>, buffer: &mut Vec<u32>, use_stackblur: bool) {
		let a = Instant::now();

		let (width, height) = {
			let PhysicalSize { width, height } = ctx.window().inner_size();
			(width as usize, height as usize)
		};

		let len = width * height;

		if buffer.len() < len {
			buffer.resize(len, 0);
		} else {
			unsafe { buffer.set_len(len) }
		}

		buffer.par_iter_mut().zip(0usize..len).for_each(|(place, index)| {
			let y: usize = index / width;
			let x: usize = index - (y * width);

			*place = (((x & 0xFFF) << 12) | ((y & 0xFFF) << 0)) as u32;
		});

		if width > 0 && height >= 8 {
			if use_stackblur {
				unsafe {
					stackblur::blur(
						&mut buffer[width * (height / 4)..width * (height / 4 * 3)],
						NonZeroUsize::new_unchecked(width),
						NonZeroUsize::new_unchecked(height / 4 * 2),
						NonZeroU32::new_unchecked(128)
					)
				}
			} else {
				let mut img = ImgRefMut::new(&mut buffer[..], width, height);
				//par_blur_srgb(&mut img.as_mut(), 8);
				par_simd_blur_srgb::<8>(&mut img.sub_image_mut(width / 8, height / 4, width - width / 4, height / 2), 128);
				//blur_srgb(&mut img.sub_image_mut(width / 8, height / 4, (width - width / 4) / 2, height / 2), 128);
				//blur_srgb(&mut img.sub_image_mut(width / 8 + (width - width / 4) / 2, height / 4, (width - width / 4) - (width - width / 4) / 2, height / 2), 128);
			}
		}

		let b = Instant::now();

		ctx.set_buffer(&buffer, width as u16, height as u16);

		let c = Instant::now();

		eprintln!("took {}μs to generate & {}μs to upload buffer", (b - a).as_micros(), (c - b).as_micros());
	}

	target.run_return(|event, _, flow| {
		if core::mem::replace(&mut first, false) {
			*flow = ControlFlow::Wait;
			redraw(&mut ctx, &mut buffer, use_old_stackblur);
			ctx.window().set_visible(true);
		}

		match event {
			Event::WindowEvent { event, window_id } if window_id == id => match event {
				WindowEvent::CloseRequested => *flow = ControlFlow::Exit,
				WindowEvent::KeyboardInput { input: KeyboardInput { state: ElementState::Pressed, virtual_keycode: Some(VirtualKeyCode::B), .. }, .. } => {
					use_old_stackblur = !use_old_stackblur;
					ctx.window().set_title(if use_old_stackblur { OLD_STACKBLUR_TITLE } else { STACKBLUR_ITER_TITLE });
					redraw(&mut ctx, &mut buffer, use_old_stackblur);
				}
				_ => {}
			},

			Event::RedrawRequested(window_id) if window_id == id => redraw(&mut ctx, &mut buffer, use_old_stackblur),

			_ => {}
		}
	})
}
