use std::{io, io::Read};

use zstd::Decoder;

use crate::{byte_compress::byte_decompress, palette::decode_palette, prediction::decode_prediction, Heightmap};

pub fn decode(data: &[u8], width: u32, height: u32) -> Result<(Heightmap, usize), io::Error> {
	let pixel_count = width as usize * height as usize;

	let (data, len) = decompress(data, pixel_count)?;

	let u16_size = pixel_count * 2 + 2;
	let u8_size = pixel_count + 3;

	let mut out = vec![0; pixel_count + 1];

	let ret = if data.len() == u16_size {
		unsafe {
			std::ptr::copy_nonoverlapping(data.as_ptr(), out.as_mut_ptr() as _, data.len());
			decode_prediction(out, width, height)
		}
	} else if data.len() == u8_size {
		unsafe {
			std::ptr::copy_nonoverlapping(data.as_ptr(), out.as_mut_ptr() as _, 4);
			byte_decompress(&data[4..], &mut out[2..]);
			decode_prediction(out, width, height)
		}
	} else {
		return Err(io::Error::new(io::ErrorKind::InvalidData, "Invalid data length"));
	};

	Ok((ret, len))
}

pub fn decompress(data: &[u8], pixel_count: usize) -> Result<(Vec<u8>, usize), io::Error> {
	let mut out = Vec::with_capacity(pixel_count * 2 + 2);
	let mut decoder = Decoder::with_buffer(data)?.single_frame();
	decoder.include_magicbytes(false)?;
	decoder.window_log_max(24)?;
	decoder.read_to_end(&mut out)?;
	let rest = decoder.finish();

	Ok((out, data.len() - rest.len()))
}
