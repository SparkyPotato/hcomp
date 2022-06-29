use std::collections::HashMap;

use crate::byte_compress::{u16_slice_to_u8_slice, u16_slice_to_u8_slice_mut};

pub fn transform_palette(data: &mut [u16]) -> Option<usize> {
	// Paletting is not worth it.
	if data.len() <= 512 {
		return None;
	}

	let mut map = HashMap::with_capacity(256);
	for &value in data.iter() {
		if value != 0 {
			map.insert(value, 0);
			if map.len() > 255 {
				return None;
			}
		}
	}

	// Don't have enough space to fit the palette.
	let offset = 1 + map.len();
	if offset > data.len() {
		return None;
	}

	let mut sorted: Vec<u16> = map.iter().map(|(&x, _)| x).collect();
	sorted.sort_unstable();

	let len = data.len();
	let data = u16_slice_to_u8_slice_mut(data);

	map.insert(0, 0);
	if let Some(index) = sorted.get(0) {
		*map.get_mut(index).unwrap() = 0;
	}
	// Delta compress palette.
	for i in (1..sorted.len()).rev() {
		*map.get_mut(&sorted[i]).unwrap() = (i + 1) as u8;
		sorted[i] -= sorted[i - 1];
	}

	let palette = sorted;

	for i in (0..len).rev() {
		let value = u16::from_le_bytes(data[2 * i..2 * i + 2].try_into().unwrap());
		data[len + i] = map[&value];
	}

	data[..offset].copy_from_slice(u16_slice_to_u8_slice(&palette));
	unsafe {
		std::ptr::copy(data[len..].as_ptr(), data[offset..].as_ptr() as _, len);
	}

	Some(offset + len)
}

pub fn decode_palette() {}

#[cfg(never)]
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
