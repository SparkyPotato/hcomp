use std::{io, io::ErrorKind};

use crate::Heightmap;

/// Result after prediction.
pub struct PredictionResult<T> {
	/// The value of the first pixel.
	pub first: u16,
	/// The minimum delta between the prediction and actual value.
	pub min_delta: i16,
	/// The deltas between the prediction and actual value - `min_delta`. The first pixel is not included.
	pub deltas_from_minimum: Vec<T>,
}

/// Transform the image into deltas from predictions.
///
/// The returned value contains the value of the first pixel, followed by the smallest delta from the prediction
/// bitcasted to a u16, followed by the rest of the deltas - minimum.
pub fn transform_prediction(data: Heightmap) -> Result<PredictionResult<u16>, io::Error> {
	let width = data.width as usize;
	let height = data.height as usize;

	let mut deltas = vec![0; data.data.len() - 1];
	let mut min_delta = i32::MAX;
	let mut max_delta = i32::MIN;

	// Predict (1, 0) and (0, 1).
	let pred = predict_none(data.data[0]);
	let delta = data.data[1] as i32 - pred;
	min_delta = min_delta.min(delta);
	max_delta = max_delta.max(delta);
	deltas[0] = delta as i16 as u16; // Truncate, we'll catch the overflow later.

	let delta = data.data[width] as i32 - pred;
	min_delta = min_delta.min(delta);
	max_delta = max_delta.max(delta);
	deltas[width - 1] = delta as i16 as u16; // ^

	// Predict the first row and column, except for (1, 0) and (0, 1).
	for x in 2..width {
		let previous_previous = data.data[x - 2];
		let previous = data.data[x - 1];

		let actual = data.data[x] as i32;
		let prediction = predict_linear(previous, previous_previous);
		let delta = actual - prediction;
		min_delta = min_delta.min(delta);
		max_delta = max_delta.max(delta);

		deltas[x - 1] = delta as i16 as u16; // ^
	}
	for y in 2..height {
		let previous_previous = data.data[(y - 2) * width];
		let previous = data.data[(y - 1) * width];

		let actual = data.data[y * width] as i32;
		let prediction = predict_linear(previous, previous_previous);
		let delta = actual - prediction;
		min_delta = min_delta.min(delta);
		max_delta = max_delta.max(delta);

		deltas[y * width - 1] = delta as i16 as u16; // ^
	}

	// Predict the sub-image that doesn't include the first row and column.
	for x in 1..width {
		for y in 1..height {
			let left = data.data[y * width + x - 1];
			let top = data.data[(y - 1) * width + x];
			let top_left = data.data[(y - 1) * width + x - 1];

			let actual = data.data[y * width + x] as i32;
			let prediction = predict_plane(left, top, top_left);
			let delta = actual - prediction;
			min_delta = min_delta.min(delta);
			max_delta = max_delta.max(delta);

			deltas[y * width + x - 1] = delta as i16 as u16; // ^
		}
	}

	// Check for over/underflow.
	let min_delta: Result<i16, _> = min_delta.try_into();
	let max_delta: Result<i16, _> = max_delta.try_into();
	match min_delta {
		Ok(min_delta) => match max_delta {
			Ok(_) => {
				// Calculate deltas from minimum.
				for value in deltas.iter_mut() {
					*value = (*value as i16 - min_delta) as u16;
				}
				Ok(PredictionResult {
					first: data.data[0],
					min_delta,
					deltas_from_minimum: deltas,
				})
			},
			Err(_) => Err(io::Error::new(ErrorKind::InvalidData, "variance too high")),
		},
		Err(_) => Err(io::Error::new(ErrorKind::InvalidData, "variance too high")),
	}
}

pub fn decode_prediction(mut data: PredictionResult<u16>, width: u32, height: u32) -> Heightmap<'static> {
	let width = width as usize;
	let height = height as usize;

	for i in data.deltas_from_minimum.iter_mut() {
		*i = (*i as i16 + data.min_delta) as u16;
	}

	let deltas = data.deltas_from_minimum;
	let mut out = vec![0; width * height];
	out[0] = data.first;
	out[1] = (predict_none(out[0]) + deltas[0] as i16 as i32) as u16;
	out[width] = (predict_none(out[0]) + deltas[width - 1] as i16 as i32) as u16;
	for x in 2..width {
		let previous = out[x - 1];
		let previous_previous = out[x - 2];
		let pred = predict_linear(previous, previous_previous);
		out[x] = (pred + deltas[x - 1] as i16 as i32) as u16;
	}
	for y in 2..height {
		let previous = out[(y - 1) * width];
		let previous_previous = out[(y - 2) * width];
		let pred = predict_linear(previous, previous_previous);
		out[y * width] = (pred + deltas[y * width - 1] as i16 as i32) as u16;
	}
	for x in 1..width {
		for y in 1..height {
			let left = out[y * width + x - 1];
			let top = out[(y - 1) * width + x];
			let top_left = out[(y - 1) * width + x - 1];
			let pred = predict_plane(left, top, top_left);
			out[y * width + x] = (pred + deltas[y * width + x - 1] as i16 as i32) as u16;
		}
	}

	Heightmap {
		width: width as u32,
		height: height as u32,
		data: out.into(),
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
	use std::borrow::Cow;

	use super::*;

	#[test]
	fn flat() {
		let values = vec![200; 5 * 5];

		let compressed = transform_prediction(Heightmap {
			width: 5,
			height: 5,
			data: Cow::Borrowed(&values),
		})
		.unwrap();
		assert_eq!(compressed.first, 200);
		assert_eq!(compressed.min_delta, 0);
		assert!(compressed.deltas_from_minimum.iter().all(|&x| x == 0));

		let decompressed = decode_prediction(compressed, 5, 5);
		assert_eq!(decompressed.data, values);
	}

	#[test]
	fn random() {
		let values = vec![
			69, 420, 47, 24, 37, 14, 108, 1645, 29, 74, 36, 197, 978, 1000, 999, 1, 0, 60, 20, 13, 8, 4, 265, 76, 23,
		];

		let compressed = transform_prediction(Heightmap {
			width: 5,
			height: 5,
			data: Cow::Borrowed(&values),
		})
		.unwrap();
		let decompressed = decode_prediction(compressed, 5, 5);
		assert_eq!(decompressed.data, values);
	}
}
