//! Plane prediction.
//! Returns the signed deltas from the predicted values.
//! The first value remains the raw height as it can't be predicted.

use std::{io, io::ErrorKind};

use crate::Heightmap;

pub fn transform_prediction(mut data: Vec<u16>, width: u32, height: u32) -> Result<Vec<i16>, io::Error> {
	let width = width as usize;
	let height = height as usize;
	assert_eq!(data.len(), width * height);

	let mut min_delta = i32::MAX;
	let mut max_delta = i32::MIN;

	// Predict the sub-image that doesn't include the first row and column.
	for x in (1..width).rev() {
		for y in (1..height).rev() {
			let left = data[y * width + x - 1] as i16;
			let top = data[(y - 1) * width + x] as i16;
			let top_left = data[(y - 1) * width + x - 1] as i16;

			let actual = data[y * width + x] as i32;
			let prediction = predict_plane(left, top, top_left);
			let delta = actual - prediction;
			min_delta = min_delta.min(delta);
			max_delta = max_delta.max(delta);

			data[y * width + x] = delta as i16 as u16; // Truncate, we'll catch the overflow later.
		}
	}

	// Predict the first row and column, except for (1, 0) and (0, 1).
	for x in (2..width).rev() {
		let previous_previous = data[x - 2] as i16;
		let previous = data[x - 1] as i16;

		let actual = data[x] as i32;
		let prediction = predict_linear(previous, previous_previous);
		let delta = actual - prediction;
		min_delta = min_delta.min(delta);
		max_delta = max_delta.max(delta);

		data[x] = delta as i16 as u16; // ^
	}
	for y in (2..height).rev() {
		let previous_previous = data[(y - 2) * width] as i16;
		let previous = data[(y - 1) * width] as i16;

		let actual = data[y * width] as i32;
		let prediction = predict_linear(previous, previous_previous);
		let delta = actual - prediction;
		min_delta = min_delta.min(delta);
		max_delta = max_delta.max(delta);

		data[y * width] = delta as i16 as u16; // ^
	}

	// Predict (1, 0) and (0, 1).
	let pred = predict_none(data[0] as i16);
	let delta = data[1] as i32 - pred;
	min_delta = min_delta.min(delta);
	max_delta = max_delta.max(delta);
	data[1] = delta as i16 as u16; // ^

	let delta = data[width] as i32 - pred;
	min_delta = min_delta.min(delta);
	max_delta = max_delta.max(delta);
	data[width] = delta as i16 as u16; // ^

	// Check for over/underflow.
	let min_d: Result<i16, _> = min_delta.try_into();
	let max_d: Result<i16, _> = max_delta.try_into();
	match (min_d, max_d) {
		(Ok(_), Ok(_)) => Ok(data.into_iter().map(|x| x as i16).collect()),
		(Err(_), Err(_)) => Err(io::Error::new(
			ErrorKind::InvalidData,
			format!(
				"variance too high: max delta is {}, min delta is {}",
				max_delta, min_delta
			),
		)),
		(Err(_), _) => Err(io::Error::new(
			ErrorKind::InvalidData,
			format!("variance too high: min delta is {}", min_delta),
		)),
		(_, Err(_)) => Err(io::Error::new(
			ErrorKind::InvalidData,
			format!("variance too high: max delta is {}", max_delta),
		)),
	}
}

pub fn decode_prediction(mut data: Vec<i16>, width: u32, height: u32) -> Heightmap<'static> {
	let width = width as usize;
	let height = height as usize;
	assert_eq!(data.len(), width * height);

	data[1] = (predict_none(data[0]) + data[1] as i32) as i16;
	data[width] = (predict_none(data[0]) + data[width] as i32) as i16;
	for x in 2..width {
		let previous = data[x - 1];
		let previous_previous = data[x - 2];
		let pred = predict_linear(previous, previous_previous);
		data[x] = (pred + data[x] as i32) as i16;
	}
	for y in 2..height {
		let previous = data[(y - 1) * width];
		let previous_previous = data[(y - 2) * width];
		let pred = predict_linear(previous, previous_previous);
		data[y * width] = (pred + data[y * width] as i32) as i16;
	}
	for x in 1..width {
		for y in 1..height {
			let left = data[y * width + x - 1];
			let top = data[(y - 1) * width + x];
			let top_left = data[(y - 1) * width + x - 1];
			let pred = predict_plane(left, top, top_left);
			data[y * width + x] = (pred + data[y * width + x] as i32) as i16;
		}
	}

	Heightmap {
		width: width as u32,
		height: height as u32,
		// Should be optimized out, as above.
		data: data.into_iter().map(|x| x as u16).collect(),
	}
}

pub fn predict_none(previous: i16) -> i32 { previous as _ }

pub fn predict_linear(previous: i16, previous_previous: i16) -> i32 {
	let delta = previous as i32 - previous_previous as i32;
	previous as i32 + delta
}

pub fn predict_plane(left: i16, top: i16, top_left: i16) -> i32 {
	let dhdy = left as i32 - top_left as i32;
	top as i32 + dhdy
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn flat() {
		let values = vec![200; 5 * 5];

		let compressed = transform_prediction(values.clone(), 5, 5).unwrap();

		assert_eq!(compressed[0], 200);
		assert!(compressed[1..].iter().all(|&x| x == 0));

		let decompressed = decode_prediction(compressed, 5, 5);
		assert_eq!(decompressed.data, values);
	}

	#[test]
	fn random() {
		let values = vec![
			69, 420, 47, 24, 37, 14, 108, 1645, 29, 74, 36, 197, 978, 1000, 999, 1, 0, 60, 20, 13, 8, 4, 265, 76, 23,
		];

		let compressed = transform_prediction(values.clone(), 5, 5).unwrap();
		let decompressed = decode_prediction(compressed, 5, 5);
		assert_eq!(decompressed.data, values);
	}
}

