use rustdct::DctPlanner;

fn transpose(mut data: Vec<f32>, width: u32, height: u32) -> Vec<f32> {
	if width == height {
		for r in 0..height {
			for c in 0..width {
				data.swap((width * r + c) as usize, (height * c + r) as usize);
			}
		}
		data
	} else {
		let mut out = vec![0.0; (width * height) as usize];
		for r in 0..height {
			for c in 0..width {
				out[(height * c + r) as usize] = data[(width * r + c) as usize]
			}
		}
		out
	}
}

fn dct_2d(mut data: Vec<f32>, width: u32, height: u32) -> Vec<f32> {
	let mut planner = DctPlanner::new();
	let w = planner.plan_dct2(width as _);
	let h = planner.plan_dct2(height as _);
	let mut s = vec![0.0; w.get_scratch_len().max(h.get_scratch_len())];

	let norm = (2.0 / width as f32).sqrt();
	for row in data.chunks_mut(width as _) {
		w.process_dct2_with_scratch(row, &mut s);
		for f in row {
			*f *= norm;
		}
	}
	let mut data = transpose(data, width, height);
	let norm = (2.0 / height as f32).sqrt();
	for col in data.chunks_mut(height as _) {
		h.process_dct2_with_scratch(col, &mut s);
		for f in col {
			*f *= norm;
		}
	}

	data
}

fn idct_2d(mut data: Vec<f32>, width: u32, height: u32) -> Vec<f32> {
	let mut planner = DctPlanner::new();
	let w = planner.plan_dct3(width as _);
	let h = planner.plan_dct3(height as _);
	let mut s = vec![0.0; w.get_scratch_len().max(h.get_scratch_len())];

	let norm = (2.0 / height as f32).sqrt();
	for col in data.chunks_mut(height as _) {
		h.process_dct3_with_scratch(col, &mut s);
		for f in col {
			*f *= norm;
		}
	}
	let mut data = transpose(data, width, height);
	let norm = (2.0 / width as f32).sqrt();
	for row in data.chunks_mut(width as _) {
		w.process_dct3_with_scratch(row, &mut s);
		for f in row {
			*f *= norm;
		}
	}

	data
}

const FACTOR: u32 = 1800;

pub fn encode_dct(mut data: Vec<i16>, width: u32, height: u32) -> (Vec<i16>, Vec<f32>) {
	let float: Vec<_> = data.iter().map(|&x| x as f32).collect();
	let mut dct = dct_2d(float, width, height);
	let w = width / FACTOR;
	let h = height / FACTOR;
	for r in h..height {
		for c in 0..width {
			dct[(height * c + r) as usize] = 0.0;
		}
	}
	for c in w..width {
		for r in 0..h {
			dct[(height * c + r) as usize] = 0.0;
		}
	}

	let inv = idct_2d(dct.clone(), width, height);
	for (x, h) in inv.into_iter().zip(data.iter_mut()) {
		*h -= x.ceil() as i16;
	}

	for r in 1..h {
		let s = (r * width) as usize;
		dct.copy_within(s..(s + w as usize), (r * w) as usize);
	}
	dct.truncate((w * h) as usize);

	(data, dct)
}

pub fn decode_dct(mut data: Vec<i16>, dct: Vec<f32>, width: u32, height: u32) -> Vec<i16> {
	let w = width / FACTOR;
	let h = height / FACTOR;

	let mut dctf = vec![0.0; (width * height) as usize];
	for r in 0..h {
		for c in 0..w {
			dctf[(height * c + r) as usize] = dct[(h * c + r) as usize];
		}
	}

	let inv = idct_2d(dctf, width, height);
	for (x, h) in inv.into_iter().zip(data.iter_mut()) {
		*h += x.ceil() as i16;
	}

	data
}

