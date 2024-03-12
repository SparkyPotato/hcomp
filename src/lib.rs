//! A library for heightmap compression.

use std::{
	borrow::Cow,
	io::{self, Write},
};

use crate::{
	dct::encode_dct,
	delta::{delta_decode, delta_encode},
	entropy::entropy_encode,
	prediction::{decode_prediction, transform_prediction},
};

pub mod dct;
pub mod delta;
pub mod entropy;
pub mod prediction;

use dct::decode_dct;
pub use entropy::compressed_len;
use entropy::entropy_decode;

/// A heightmap image.
///
/// `data` represents the heights of each pixel, in row-major order.
/// `data.len()` must be equal to `width * height`.
#[derive(Clone)]
pub struct Heightmap<'a> {
	pub width: u32,
	pub height: u32,
	pub data: Cow<'a, [u16]>,
}

/// Encode a heightmap.
pub fn encode(heightmap: Heightmap, output: &mut impl Write) -> Result<usize, io::Error> {
	assert!(
		heightmap.width > 2 && heightmap.height > 2,
		"Heightmap must be at least 3x3"
	);
	assert_eq!(
		heightmap.data.len(),
		heightmap.width as usize * heightmap.height as usize,
		"heightmap data length must be equal to width * height"
	);

	let predicted = transform_prediction(heightmap.data.into(), heightmap.width, heightmap.height)?;
	// let (data, dct) = encode_dct(predicted, heightmap.width, heightmap.height);
	let deltas = delta_encode(predicted);
	entropy_encode(&[], &deltas, heightmap.width, heightmap.height, output)
}

/// Decode a heightmap.
pub fn decode(data: &[u8], width: u32, height: u32) -> Result<(Heightmap<'static>, usize), io::Error> {
	let (dct, deltas, len) = entropy_decode(data, width, height)?;
	let data = delta_decode(deltas);
	// let predicted = decode_dct(data, dct, width, height);
	let ret = decode_prediction(data, width, height);
	Ok((ret, len))
}

#[cfg(test)]
mod tests {
	use std::borrow::Cow;

	use crate::*;

	#[test]
	fn flat() {
		let values = vec![200; 4 * 4];
		let mut output = Vec::new();
		encode(
			Heightmap {
				width: 4,
				height: 4,
				data: Cow::Borrowed(&values),
			},
			&mut output,
		)
		.unwrap();
		let (decompressed, _) = decode(&output, 4, 4).unwrap();
		assert_eq!(decompressed.data, values);
	}

	#[test]
	fn random() {
		let values = vec![69, 420, 47, 24, 37, 14, 108, 1645, 29, 74, 36, 197, 978, 1000, 999, 1];

		let mut output = Vec::new();
		encode(
			Heightmap {
				width: 4,
				height: 4,
				data: Cow::Borrowed(&values),
			},
			&mut output,
		)
		.unwrap();
		let (decompressed, _) = decode(&output, 4, 4).unwrap();
		assert_eq!(decompressed.data, values);
	}
}

