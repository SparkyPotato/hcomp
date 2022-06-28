//! A library for heightmap compression.

use std::borrow::Cow;

pub mod decode;
pub mod encode;

mod byte_compress;
mod palette;
mod prediction;
mod stream;

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
		let values = vec![200; 5 * 5];
		let mut output = Vec::new();
		encode(
			Heightmap {
				width: 5,
				height: 5,
				data: Cow::Borrowed(&values),
			},
			22,
			&mut output,
		)
		.unwrap();
		let (decompressed, _) = decode(&output, 5, 5).unwrap();
		assert_eq!(decompressed.data, values);
	}

	#[test]
	fn random() {
		let values = vec![
			69, 420, 47, 24, 37, 14, 108, 1645, 29, 74, 36, 197, 978, 1000, 999, 1, 0, 60, 20, 13, 8, 4, 265, 76, 23,
		];

		let mut output = Vec::new();
		encode(Heightmap {
			width: 5,
			height: 5,
			data: Cow::Borrowed(&values),
		}, 22, &mut output)
			.unwrap();
		let (decompressed, _) = decode(&output, 5, 5).unwrap();
		assert_eq!(decompressed.data, values);
	}
}
