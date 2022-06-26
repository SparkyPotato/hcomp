use std::{io, io::Read};

use zstd::Decoder;

use crate::{
	byte_compress::decode_byte_compress,
	palette::decode_palette,
	prediction::decode_prediction,
	stream::{decode_stream, max_size_for_pixel_count},
	Heightmap,
};

pub fn decode(data: &[u8], width: u32, height: u32) -> Result<(Heightmap, usize), io::Error> {
	let pixel_count = width as usize * height as usize;
	let (stream, len) = decompress(data, pixel_count)?;
	let byte_compressed = decode_stream(&stream, pixel_count)?;
	let paletted = decode_byte_compress(byte_compressed);
	let predicted = decode_palette(paletted);
	Ok((decode_prediction(predicted, width, height), len))
}

pub fn decompress(data: &[u8], pixel_count: usize) -> Result<(Vec<u8>, usize), io::Error> {
	let mut out = Vec::with_capacity(max_size_for_pixel_count(pixel_count));
	let mut decoder = Decoder::with_buffer(data)?.single_frame();
	decoder.include_magicbytes(false)?;
	decoder.window_log_max(24)?;
	decoder.read_to_end(&mut out)?;
	let rest = decoder.finish();

	Ok((out, data.len() - rest.len()))
}
