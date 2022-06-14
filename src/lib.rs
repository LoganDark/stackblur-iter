use std::collections::VecDeque;
use std::iter::Peekable;

pub struct StackBlur<I: Iterator<Item = usize>> {
	iter: Peekable<I>,
	radius: usize,
	ops: VecDeque<isize>,
	to_output: usize,
	dnom: usize
}

impl<I: Iterator<Item = usize>> StackBlur<I> {
	pub fn new(mut iter: I, radius: usize, mut ops: VecDeque<usize>) -> Self {
		ops.clear();

		let mut dnom = 0;

		for i in 0..=radius {
			let item = match iter.next() {
				Some(item) => item,
				None => break
			};

			let mul = radius + 1 - i;
			stack.push_back(item * mul);
			dnom += mul;
		}

		Self { iter: iter.peekable(), radius, ops: stack, to_output: 0, dnom }
	}

	pub fn into_stack(self) -> VecDeque<usize> {
		self.lops
	}
}

impl<I: Iterator<Item = usize>> Iterator for StackBlur<I> {
	type Item = usize;

	fn next(&mut self) -> Option<Self::Item> {
		if self.dnom == 0 || self.to_output >= self.lops.len() { return None }

		let result = self.lops.iter().sum::<usize>() / self.dnom;

		// lop off and append right
		for (item, i) in self.lops.iter_mut().zip(0usize..) {
			let mul = self.radius + 1 - self.to_output.abs_diff(i);

			if i <= self.to_output {
				*item -= *item / mul;
				self.dnom -= 1;
			} else {
				*item += *item / mul;
				self.dnom += 1;
			}
		}

		// Idk lol
		if self.to_output == self.radius {
			self.lops.pop_front();
		} else {
			self.to_output += 1;
		}

		if self.lops.len() - self.to_output <= self.radius && self.iter.peek().is_some() {
			self.lops.push_back(self.iter.next().unwrap());
			self.dnom += 1;
		}

		Some(result)
	}
}

pub fn blur_horiz(argb: &mut [u32], width: usize, height: usize, radius: usize) {
	debug_assert_eq!(argb.len(), width * height);

	let mut stack_r = VecDeque::new();
	let mut stack_g = VecDeque::new();
	let mut stack_b = VecDeque::new();

	for row in argb.chunks_exact_mut(width) {
		let not_safe = row as *mut [u32];

		let read_r = unsafe { (*not_safe).iter() }.copied().map(|i| i.to_be_bytes()[1] as usize);
		let read_g = unsafe { (*not_safe).iter() }.copied().map(|i| i.to_be_bytes()[2] as usize);
		let read_b = unsafe { (*not_safe).iter() }.copied().map(|i| i.to_be_bytes()[3] as usize);

		let mut iter_r = StackBlur::new(read_r, radius, stack_r);
		let mut iter_g = StackBlur::new(read_g, radius, stack_g);
		let mut iter_b = StackBlur::new(read_b, radius, stack_b);

		let mut index = 0usize;
		while let (Some(r), Some(g), Some(b)) = (iter_r.next(), iter_g.next(), iter_b.next()) {
			unsafe { (*not_safe)[index] = u32::from_be_bytes([255, r as u8, g as u8, b as u8]) };
			index += 1;
		}

		stack_r = iter_r.into_stack();
		stack_g = iter_g.into_stack();
		stack_b = iter_b.into_stack();
	}
}

pub fn blur_vert(argb: &mut [u32], width: usize, height: usize, radius: usize) {
	debug_assert_eq!(argb.len(), width * height);

	let mut stack_r = VecDeque::new();
	let mut stack_g = VecDeque::new();
	let mut stack_b = VecDeque::new();

	for col in 0..width {
		let not_safe = argb as *mut [u32];

		let read_r = unsafe { (*not_safe).iter() }.skip(col).step_by(width).copied().map(|i| i.to_be_bytes()[1] as usize);
		let read_g = unsafe { (*not_safe).iter() }.skip(col).step_by(width).copied().map(|i| i.to_be_bytes()[2] as usize);
		let read_b = unsafe { (*not_safe).iter() }.skip(col).step_by(width).copied().map(|i| i.to_be_bytes()[3] as usize);

		let mut iter_r = StackBlur::new(read_r, radius, stack_r);
		let mut iter_g = StackBlur::new(read_g, radius, stack_g);
		let mut iter_b = StackBlur::new(read_b, radius, stack_b);

		let mut row = 0usize;
		while let (Some(r), Some(g), Some(b)) = (iter_r.next(), iter_g.next(), iter_b.next()) {
			unsafe { (*not_safe)[row * width + col] = u32::from_be_bytes([255, r as u8, g as u8, b as u8]) };
			row += 1;
		}

		stack_r = iter_r.into_stack();
		stack_g = iter_g.into_stack();
		stack_b = iter_b.into_stack();
	}
}

pub fn blur(argb: &mut [u32], width: usize, height: usize, radius: usize) {
	blur_horiz(argb, width, height, radius);
	blur_vert(argb, width, height, radius);
}
