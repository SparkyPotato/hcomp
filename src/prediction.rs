use std::{io, io::ErrorKind};

use crate::Heightmap;

/// Transform the image into deltas from predictions.
/// * `[0]`: first value in the heightmap.
/// * `[1]`: minimum delta from a prediction.
/// * `[2..]`: deltas from prediction - min_delta for the rest of the pixels.
pub fn transform_prediction(mut data: Vec<u16>, width: u32, height: u32) -> Result<Vec<u16>, io::Error> {
	let width = width as usize;
	let height = height as usize;
	assert_eq!(data.len(), width * height);

	let mut min_delta = i32::MAX;
	let mut max_delta = i32::MIN;

	// Predict the sub-image that doesn't include the first row and column.
	for x in (1..width).rev() {
		for y in (1..height).rev() {
			let left = data[y * width + x - 1];
			let top = data[(y - 1) * width + x];
			let top_left = data[(y - 1) * width + x - 1];

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
		let previous_previous = data[x - 2];
		let previous = data[x - 1];

		let actual = data[x] as i32;
		let prediction = predict_linear(previous, previous_previous);
		let delta = actual - prediction;
		min_delta = min_delta.min(delta);
		max_delta = max_delta.max(delta);

		data[x] = delta as i16 as u16; // ^
	}
	for y in (2..height).rev() {
		let previous_previous = data[(y - 2) * width];
		let previous = data[(y - 1) * width];

		let actual = data[y * width] as i32;
		let prediction = predict_linear(previous, previous_previous);
		let delta = actual - prediction;
		min_delta = min_delta.min(delta);
		max_delta = max_delta.max(delta);

		data[y * width] = delta as i16 as u16; // ^
	}

	// Predict (1, 0) and (0, 1).
	let pred = predict_none(data[0]);
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
	match min_d {
		Ok(min_d) => {
			if (max_delta - min_delta) as u32 <= u16::MAX as u32 {
				// Calculate deltas from minimum.
				data.push(0);
				for i in (1..data.len() - 1).rev() {
					data[i + 1] = (data[i] as i16 - min_d) as u16;
				}
				data[1] = min_d as u16;
				Ok(data)
			} else {
				Err(io::Error::new(
					ErrorKind::InvalidData,
					format!("variance too high: max delta is {}", max_delta),
				))
			}
		},
		Err(_) => Err(io::Error::new(
			ErrorKind::InvalidData,
			format!("variance too high: min delta is {}", min_delta),
		)),
	}
}

pub fn decode_prediction(mut data: Vec<u16>, width: u32, height: u32) -> Heightmap<'static> {
	let width = width as usize;
	let height = height as usize;
	assert_eq!(data.len(), width * height + 1);

	let min_delta = data[1] as i16;
	for i in 2..data.len() {
		data[i - 1] = (data[i] as i16 + min_delta) as u16;
	}
	data.truncate(data.len() - 1);

	data[1] = (predict_none(data[0]) + data[1] as i16 as i32) as u16;
	data[width] = (predict_none(data[0]) + data[width] as i16 as i32) as u16;
	for x in 2..width {
		let previous = data[x - 1];
		let previous_previous = data[x - 2];
		let pred = predict_linear(previous, previous_previous);
		data[x] = (pred + data[x] as i16 as i32) as u16;
	}
	for y in 2..height {
		let previous = data[(y - 1) * width];
		let previous_previous = data[(y - 2) * width];
		let pred = predict_linear(previous, previous_previous);
		data[y * width] = (pred + data[y * width] as i16 as i32) as u16;
	}
	for x in 1..width {
		for y in 1..height {
			let left = data[y * width + x - 1];
			let top = data[(y - 1) * width + x];
			let top_left = data[(y - 1) * width + x - 1];
			let pred = predict_plane(left, top, top_left);
			data[y * width + x] = (pred + data[y * width + x] as i16 as i32) as u16;
		}
	}

	Heightmap {
		width: width as u32,
		height: height as u32,
		data: data.into(),
	}
}

fn predict_none(previous: u16) -> i32 { previous as _ }

fn predict_linear(previous: u16, previous_previous: u16) -> i32 {
	let delta = previous as i32 - previous_previous as i32;
	previous as i32 + delta
}

fn predict_plane(left: u16, top: u16, top_left: u16) -> i32 {
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
