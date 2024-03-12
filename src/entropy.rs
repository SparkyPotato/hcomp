//! WebP and zstd based entropy coding.
//!
//! Stores data in the R and G subpixels.
//! The decompressed input should have width * height + 1 pixels, and the first is stored
//! uncompressed.

use std::{
	io::{self, Read, Write},
	slice,
};

use libwebp_sys::{
	WebPDecodeRGBAInto,
	WebPEncode,
	WebPInitConfig,
	WebPPicture,
	WebPPictureFree,
	WebPPictureImportRGBA,
	WebPPictureInit,
};
use zstd::{Decoder, Encoder};

struct WCtx<'a, T: Write> {
	w: &'a mut T,
	b: usize,
	err: Result<(), io::Error>,
}

impl<T: Write> Write for WCtx<'_, T> {
	fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
		let s = self.w.write(buf)?;
		self.b += s;
		Ok(s)
	}

	fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
		let s = self.w.write_vectored(bufs)?;
		self.b += s;
		Ok(s)
	}

	fn flush(&mut self) -> io::Result<()> { self.w.flush() }
}

pub fn entropy_encode<T: Write>(
	dct: &[f32], data: &[u16], width: u32, height: u32, output: &mut T,
) -> Result<usize, io::Error> {
	unsafe {
		let writer = WCtx {
			w: output,
			b: 0,
			err: Ok(()),
		};

		let mut enc = Encoder::new(writer, 11)?;
		enc.write_all(slice::from_raw_parts(dct.as_ptr() as _, dct.len() * 4))?;
		let mut writer = enc.finish()?;

		let mut config = std::mem::zeroed();
		WebPInitConfig(&mut config);
		config.lossless = 1;
		config.quality = 100.0;
		config.method = 4;

		let mut picture = std::mem::zeroed();
		WebPPictureInit(&mut picture);
		picture.use_argb = 1;
		picture.writer = Some(write::<T>);
		picture.custom_ptr = &mut writer as *mut _ as _;
		picture.width = width as i32;
		picture.height = height as i32;

		writer.write(&data[0].to_le_bytes())?;

		let mut pad = Vec::with_capacity(data.len() - 1);
		for &d in data[1..].iter() {
			let b = d.to_le_bytes();
			pad.push(u32::from_le_bytes([b[0], b[1], 0, 255]));
		}

		WebPPictureImportRGBA(&mut picture, pad.as_ptr() as _, width as i32 * 4);
		WebPEncode(&config, &mut picture);
		WebPPictureFree(&mut picture);

		if picture.error_code as i32 != 0 {
			panic!("WebPEncode failed: {}", picture.error_code as i32)
		}

		unsafe extern "C" fn write<T: Write>(data: *const u8, data_size: usize, picture: *const WebPPicture) -> i32 {
			let c = &mut *((*picture).custom_ptr as *mut WCtx<T>);
			if let Err(e) = c.write(slice::from_raw_parts(data, data_size)) {
				c.err = Err(e);
			}
			1
		}

		writer.err.map(|_| writer.b)
	}
}

pub fn compressed_len(data: &[u8]) -> u32 { u32::from_le_bytes(data[6..10].try_into().unwrap()) + 10 }

pub fn entropy_decode(data: &[u8], width: u32, height: u32) -> Result<(Vec<f32>, Vec<u16>, usize), io::Error> {
	let mut dec = Decoder::with_buffer(data)?.single_frame();
	let mut dct = Vec::new();
	dec.read_to_end(&mut dct)?;
	let dct = dct
		.chunks_exact(4)
		.map(|x| f32::from_ne_bytes(x.try_into().unwrap()))
		.collect();
	let old = data;
	let data = dec.finish();
	let len = old.len() - data.len();

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
			return Err(io::Error::new(
				io::ErrorKind::InvalidData,
				"Failed to decode webp container",
			));
		}
		dec.set_len(dec.capacity());
		decompressed.extend(dec.into_iter().step_by(2))
	};
	Ok((dct, decompressed, compressed_len(data) as usize + len))
}

