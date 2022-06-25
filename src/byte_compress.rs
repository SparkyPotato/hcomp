use crate::{palette::PaletteResult, prediction::PredictionResult};

pub enum ByteCompressResult {
	Compressed(PaletteResult<u8>),
	Uncompressed(PaletteResult<u16>),
}

pub fn byte_compress(data: PaletteResult<u16>) -> ByteCompressResult {
	match data {
		PaletteResult::Unpalleted(data) => {
			let mut max_delta = 0;
			for &delta in data.deltas_from_minimum.iter() {
				max_delta = max_delta.max(delta);
			}
			if max_delta <= u8::MAX as u16 {
				ByteCompressResult::Compressed(PaletteResult::Unpalleted(PredictionResult {
					first: data.first,
					min_delta: data.min_delta,
					deltas_from_minimum: data.deltas_from_minimum.iter().map(|&delta| delta as u8).collect(),
				}))
			} else {
				ByteCompressResult::Uncompressed(PaletteResult::Unpalleted(data))
			}
		},
		PaletteResult::Palleted {
			first,
			min_delta,
			palette,
			data,
		} => {
			let mut max_delta = 0;
			for &delta in palette.iter() {
				max_delta = max_delta.max(delta);
			}
			if max_delta <= u8::MAX as u16 {
				ByteCompressResult::Compressed(PaletteResult::Palleted {
					first,
					min_delta,
					palette: palette.iter().map(|&delta| delta as u8).collect(),
					data,
				})
			} else {
				ByteCompressResult::Uncompressed(PaletteResult::Palleted {
					first,
					min_delta,
					palette,
					data,
				})
			}
		},
	}
}

pub fn decode_byte_compress(data: ByteCompressResult) -> PaletteResult<u16> {
	match data {
		ByteCompressResult::Uncompressed(data) => data,
		ByteCompressResult::Compressed(PaletteResult::Unpalleted(PredictionResult {
			first,
			min_delta,
			deltas_from_minimum,
		})) => PaletteResult::Unpalleted(PredictionResult {
			first,
			min_delta,
			deltas_from_minimum: deltas_from_minimum.into_iter().map(|x| x as u16).collect(),
		}),
		ByteCompressResult::Compressed(PaletteResult::Palleted {
			first,
			min_delta,
			palette,
			data,
		}) => PaletteResult::Palleted {
			first,
			min_delta,
			palette: palette.into_iter().map(|x| x as u16).collect(),
			data,
		},
	}
}
