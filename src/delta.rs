//! Unsigned delta encoding.
//!
//! Signed numbers are converted to unsigned deltas from a minimum delta.
//! The first number is not considered.
//!
//! - [0]: First number in the input (cast to u16).
//! - [1]: Minimum value in the rest.
//! - [2..]: Deltas from this minimum value.

pub fn delta_encode(mut data: Vec<i16>) -> Vec<u16> {
	let min = data[1..].iter().copied().min().unwrap();
	data.push(0);
	let mut last = min;
	for v in data[1..].iter_mut() {
		let d = *v - min;
		*v = last;
		last = d;
	}
	// This should be optimized out.
	data.into_iter().map(|x| x as u16).collect()
}

pub fn delta_decode(mut data: Vec<u16>) -> Vec<i16> {
	let min = data[1] as i16;
	for i in 1..(data.len() - 1) {
		data[i] = (data[i + 1] as i16 + min) as u16;
	}
	data.pop();
	// This should be optimized out.
	data.into_iter().map(|x| x as i16).collect()
}

