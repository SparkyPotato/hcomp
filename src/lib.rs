//! A library for heightmap compression.

use std::borrow::Cow;

pub mod decode;
pub mod encode;

mod prediction;

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

#[cfg(test)]
mod tests {
	use std::borrow::Cow;

	use crate::{decode::decode, encode::encode, Heightmap};

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
			22,
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
			22,
			&mut output,
		)
		.unwrap();
		let (decompressed, _) = decode(&output, 4, 4).unwrap();
		assert_eq!(decompressed.data, values);
	}
}

