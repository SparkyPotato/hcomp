use std::collections::{HashMap, HashSet};

use crate::prediction::PredictionResult;

/// Result after generating global palettes.
pub enum PaletteResult<T> {
	/// No palette was generated. The input is returned unmodified.
	Unpalleted(PredictionResult<T>),
	/// A palette was generated.
	/// * `first`: The value of the first pixel, since it is not stored in the palette.
	/// * `min_delta`: The minimum delta between the prediction and actual value.
	/// * `palette`: Sorted delta-compressed palette of deltas, not including the minimum value.
	/// * `data`: Indices into the palette for each pixel after the first one. (0 signifies `min_delta`).
	Palleted {
		first: u16,
		min_delta: i16,
		palette: Vec<T>,
		data: Vec<u8>,
	},
}

pub fn transform_palette(data: PredictionResult<u16>) -> PaletteResult<u16> {
	let mut uniques = HashSet::with_capacity(256);
	for &delta in data.deltas_from_minimum.iter() {
		if delta != 0 {
			uniques.insert(delta);
			if uniques.len() > 255 {
				return PaletteResult::Unpalleted(data);
			}
		}
	}

	let mut sorted: Vec<_> = uniques.into_iter().collect();
	sorted.sort_unstable();

	let mut map = HashMap::with_capacity(256);
	map.insert(0, 0);
	if let Some(&x) = sorted.get(0) {
		map.insert(x, 1);
	}

	// Delta compress palette.
	for i in (1..sorted.len()).rev() {
		map.insert(sorted[i], (i + 1) as u8);
		sorted[i] -= sorted[i - 1];
	}

	PaletteResult::Palleted {
		first: data.first,
		min_delta: data.min_delta,
		palette: sorted,
		data: data.deltas_from_minimum.iter().map(|delta| map[delta]).collect(),
	}
}

pub fn decode_palette(data: PaletteResult<u16>) -> PredictionResult<u16> {
	match data {
		PaletteResult::Unpalleted(data) => data,
		PaletteResult::Palleted {
			first,
			min_delta,
			palette,
			data,
		} => {
			let mut palette = palette;
			for i in 1..palette.len() {
				palette[i] += palette[i - 1];
			}

			PredictionResult {
				first,
				min_delta,
				deltas_from_minimum: data
					.into_iter()
					.map(|i| if i == 0 { 0 } else { palette[(i - 1) as usize] })
					.collect(),
			}
		},
	}
}

#[cfg(test)]
mod tests {
	use std::borrow::Cow;

	use super::*;
	use crate::{
		prediction::{decode_prediction, transform_prediction},
		Heightmap,
	};

	#[test]
	fn flat_palette() {
		let compressed = transform_palette(PredictionResult {
			first: 200,
			min_delta: 0,
			deltas_from_minimum: vec![0; 24],
		});

		match &compressed {
			PaletteResult::Unpalleted(_) => panic!("Expected palette"),
			PaletteResult::Palleted {
				first,
				min_delta,
				palette,
				data,
			} => {
				assert_eq!(*first, 200);
				assert_eq!(*min_delta, 0);
				assert_eq!(palette, &[]);
				assert_eq!(data, &[0; 24]);
			},
		}

		let decompressed = decode_palette(compressed);
		assert_eq!(decompressed.first, 200);
		assert_eq!(decompressed.min_delta, 0);
		assert_eq!(decompressed.deltas_from_minimum, vec![0; 24]);
	}

	#[test]
	fn random_palette() {
		let values = vec![
			69, 420, 47, 24, 37, 14, 108, 1645, 29, 74, 36, 197, 978, 1000, 999, 1, 0, 60, 20, 13, 8, 4, 265, 76, 23,
		];

		let compressed = transform_prediction(Heightmap {
			width: 5,
			height: 5,
			data: Cow::Borrowed(&values),
		})
		.unwrap();
		let compressed = transform_palette(compressed);
		let decompressed = decode_palette(compressed);
		let decompressed = decode_prediction(decompressed, 5, 5);
		assert_eq!(decompressed.data, values);
	}
}
