use std::io;

use libwebp_sys::WebPDecodeRGBAInto;

use crate::{prediction::decode_prediction, Heightmap};

pub fn decode(data: &[u8], width: u32, height: u32) -> Result<(Heightmap, usize), io::Error> {
	let (data, len) = decompress_webp(data, width, height)?;
	let ret = decode_prediction(data, width, height);
	Ok((ret, len))
}

pub fn compressed_len(data: &[u8]) -> u32 { u32::from_le_bytes(data[6..10].try_into().unwrap()) + 10 }

fn decompress_webp(data: &[u8], width: u32, height: u32) -> Result<(Vec<u16>, usize), io::Error> {
	let mut decompressed: Vec<u16> = Vec::with_capacity(width as usize * height as usize + 1);
	decompressed.push(u16::from_le_bytes(data[0..2].try_into().unwrap()));
	let d = &data[2..];
	unsafe {
		let mut dec: Vec<u16> = Vec::with_capacity(width as usize * height as usize * 2);
		if WebPDecodeRGBAInto(
			d.as_ptr(),
			d.len(),
			dec.as_mut_ptr() as _,
			dec.capacity() * 2,
			width as i32 * 4,
		)
		.is_null()
		{
			panic!("WebPDecodeRGBAInto failed")
		}
		dec.set_len(dec.capacity());
		decompressed.extend(dec.into_iter().step_by(2))
	};
	Ok((decompressed, compressed_len(data) as usize))
}

