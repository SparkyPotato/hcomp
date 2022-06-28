use std::io;

use crate::{byte_compress::ByteCompressResult, palette::PaletteResult, prediction::PredictionResult};

pub fn max_size_for_pixel_count(pixel_count: usize) -> usize { pixel_count * 2 + 2 }

pub fn generate_stream(data: ByteCompressResult) -> Vec<u8> {
	match data {
		ByteCompressResult::Uncompressed(PaletteResult::Unpalleted(PredictionResult {
			first,
			min_delta,
			deltas_from_minimum,
		})) => std::iter::once(first.to_le_bytes())
			.flatten()
			.chain(min_delta.to_le_bytes())
			.chain(deltas_from_minimum.into_iter().flat_map(|x| x.to_le_bytes()))
			.collect(),
		ByteCompressResult::Compressed(PaletteResult::Unpalleted(PredictionResult {
			first,
			min_delta,
			deltas_from_minimum,
		})) => std::iter::once(first.to_le_bytes())
			.flatten()
			.chain(min_delta.to_le_bytes())
			.chain(deltas_from_minimum)
			.collect(),
		ByteCompressResult::Uncompressed(PaletteResult::Palleted {
			first,
			min_delta,
			palette,
			data,
		}) => std::iter::once(first.to_le_bytes())
			.flatten()
			.chain(min_delta.to_le_bytes())
			.chain(Some(palette.len() as u8))
			.chain(palette.into_iter().flat_map(|x| x.to_le_bytes()))
			.chain(data)
			.collect(),
		ByteCompressResult::Compressed(PaletteResult::Palleted {
			first,
			min_delta,
			palette,
			data,
		}) => std::iter::once(first.to_le_bytes())
			.flatten()
			.chain(min_delta.to_le_bytes())
			.chain(Some(palette.len() as u8))
			.chain(palette)
			.chain(data)
			.collect(),
	}
}

pub fn decode_stream(data: &[u8], pixel_count: usize) -> Result<ByteCompressResult, io::Error> {
	let uncompressed_size = pixel_count * 2 + 2;
	let compressed_size = pixel_count + 3;
	let first = u16::from_le_bytes(data[0..2].try_into().unwrap());
	let min_delta = i16::from_le_bytes(data[2..4].try_into().unwrap());
	let rest = &data[4..];

	if data.len() == uncompressed_size {
		Ok(ByteCompressResult::Uncompressed(PaletteResult::Unpalleted(
			PredictionResult {
				first,
				min_delta,
				deltas_from_minimum: rest
					.chunks_exact(2)
					.map(|x| u16::from_le_bytes(x.try_into().unwrap()))
					.collect(),
			},
		)))
	} else if data.len() == compressed_size {
		Ok(ByteCompressResult::Compressed(PaletteResult::Unpalleted(
			PredictionResult {
				first,
				min_delta,
				deltas_from_minimum: rest.into(),
			},
		)))
	} else {
		let palette_len = rest[0] as usize;
		let data_len = pixel_count - 1;
		let palette_byte_len = rest.len() - data_len - 1;
		if palette_byte_len == palette_len * 2 {
			Ok(ByteCompressResult::Uncompressed(PaletteResult::Palleted {
				first,
				min_delta,
				palette: rest[1..1 + palette_byte_len]
					.chunks_exact(2)
					.map(|x| u16::from_le_bytes(x.try_into().unwrap()))
					.collect(),
				data: rest[1 + palette_byte_len..].into(),
			}))
		} else if palette_byte_len == palette_len {
			Ok(ByteCompressResult::Compressed(PaletteResult::Palleted {
				first,
				min_delta,
				palette: rest[1..1 + palette_byte_len].into(),
				data: rest[1 + palette_byte_len..].into(),
			}))
		} else {
			Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid data length"))
		}
	}
}
