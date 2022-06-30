use std::{
	borrow::Cow,
	io::{Read, Write},
	time::Instant,
};

use hcomp::{decode::decode, encode::encode, Heightmap};
use libwebp_sys::{
	WebPDecodeRGBAInto,
	WebPEncode,
	WebPImageHint::WEBP_HINT_GRAPH,
	WebPInitConfig,
	WebPPicture,
	WebPPictureImportRGBA,
	WebPPictureInit,
};
use xz2::write::{XzDecoder, XzEncoder};
use zstd::{Decoder, Encoder};

use crate::output::{Compression, Size, Time};

pub fn hcomp(data: &[u16], width: u32, height: u32) -> Compression {
	print!("hcomp: compressing");
	std::io::stdout().flush().unwrap();
	let start = Instant::now();
	let mut compressed: Vec<u8> = Vec::new();
	let compress_size = encode(
		Heightmap {
			width,
			height,
			data: Cow::Borrowed(&data),
		},
		22,
		&mut compressed,
	)
	.unwrap();
	let compress_duration = start.elapsed();

	print!("\rhcomp: decompressing");
	std::io::stdout().flush().unwrap();
	let start = Instant::now();
	let (out, _) = decode(&compressed, width, height).unwrap();
	let decompress_duration = start.elapsed();

	let lossless = out.data == data;

	print!("\r");
	std::io::stdout().flush().unwrap();

	Compression {
		name: "hcomp".into(),
		size: Size::new(compress_size),
		compress: Time::new(compress_duration),
		decompress: Time::new(decompress_duration),
		lossless,
		orig_size: Size::new(data.len() * 2),
	}
}

pub fn webp(data: &[u16], width: u32, height: u32) -> Compression {
	print!("webp: compressing");
	std::io::stdout().flush().unwrap();
	let start = Instant::now();
	let compressed = unsafe {
		let mut temp: Vec<u8> = Vec::new();

		let mut config = std::mem::zeroed();
		WebPInitConfig(&mut config);
		config.lossless = 1;
		config.quality = 100.0;
		config.method = 6;
		config.image_hint = WEBP_HINT_GRAPH;
		config.exact = 1;

		let mut picture = std::mem::zeroed();
		WebPPictureInit(&mut picture);
		picture.use_argb = 1;
		picture.writer = Some(write);
		picture.custom_ptr = &mut temp as *mut _ as _;
		picture.width = width as i32 / 2;
		picture.height = height as i32;

		WebPPictureImportRGBA(&mut picture, data.as_ptr() as _, width as i32 * 2);

		WebPEncode(&config, &mut picture);

		if picture.error_code as i32 != 0 {
			panic!("WebPEncode failed: {}", picture.error_code as i32)
		}

		unsafe extern "C" fn write(data: *const u8, data_size: usize, picture: *const WebPPicture) -> i32 {
			let vec = &mut *((*picture).custom_ptr as *mut Vec<u8>);
			vec.extend_from_slice(std::slice::from_raw_parts(data, data_size));

			1
		}

		temp
	};
	let compress_duration = start.elapsed();

	print!("\rwebp: decompressing");
	std::io::stdout().flush().unwrap();
	let start = Instant::now();

	let mut decompressed: Vec<u16> = Vec::with_capacity(width as usize * height as usize);
	decompressed.resize(decompressed.capacity(), 0);

	unsafe {
		if WebPDecodeRGBAInto(
			compressed.as_ptr(),
			compressed.len(),
			decompressed.as_mut_ptr() as _,
			decompressed.len() * 2,
			width as i32 * 2,
		)
		.is_null()
		{
			panic!("WebPDecodeRGBAInto failed")
		}
	};

	let decompress_duration = start.elapsed();

	let lossless = decompressed == data;

	print!("\r");
	std::io::stdout().flush().unwrap();

	Compression {
		name: "webp".into(),
		size: Size::new(compressed.len()),
		compress: Time::new(compress_duration),
		decompress: Time::new(decompress_duration),
		lossless,
		orig_size: Size::new(data.len() * 2),
	}
}

pub fn zstd(data: &[u16]) -> Compression {
	print!("zstd: compressing");
	std::io::stdout().flush().unwrap();
	let start = Instant::now();

	let mut compressed = Vec::new();
	let mut enc = Encoder::new(&mut compressed, 22).unwrap();
	enc.set_pledged_src_size(Some(data.len() as u64 * 2)).unwrap();
	enc.window_log(24).unwrap();
	enc.write_all(unsafe { std::slice::from_raw_parts(data.as_ptr() as _, data.len() * 2) })
		.unwrap();
	enc.finish().unwrap();

	let compress_duration = start.elapsed();

	print!("\rzstd: decompressing");
	std::io::stdout().flush().unwrap();
	let start = Instant::now();

	let mut dec = Decoder::with_buffer(compressed.as_slice()).unwrap();
	dec.window_log_max(24).unwrap();
	let mut out = Vec::with_capacity(data.len());
	dec.read_to_end(&mut out).unwrap();

	let decompress_duration = start.elapsed();

	let lossless = out
		.chunks_exact(2)
		.zip(data)
		.all(|(d, &h)| h == u16::from_ne_bytes(d.try_into().unwrap()));

	print!("\r");
	std::io::stdout().flush().unwrap();

	Compression {
		name: "zstd".into(),
		size: Size::new(compressed.len()),
		compress: Time::new(compress_duration),
		decompress: Time::new(decompress_duration),
		lossless,
		orig_size: Size::new(data.len() * 2),
	}
}

pub fn xz2(data: &[u16]) -> Compression {
	print!("xz2: compressing");
	std::io::stdout().flush().unwrap();
	let start = Instant::now();
	let mut compressed = Vec::new();
	let mut enc = XzEncoder::new(&mut compressed, 9);
	enc.write_all(unsafe { std::slice::from_raw_parts(data.as_ptr() as _, data.len() * 2) })
		.unwrap();
	enc.finish().unwrap();
	let compress_duration = start.elapsed();

	print!("\rxz2: decompressing");
	std::io::stdout().flush().unwrap();
	let start = Instant::now();
	let mut out = Vec::with_capacity(data.len());
	let mut dec = XzDecoder::new(&mut out);
	dec.write_all(&compressed).unwrap();
	dec.finish().unwrap();
	drop(dec);
	let decompress_duration = start.elapsed();

	let lossless = out
		.chunks_exact(2)
		.zip(data)
		.all(|(d, &h)| h == u16::from_ne_bytes(d.try_into().unwrap()));

	print!("\r");
	std::io::stdout().flush().unwrap();

	Compression {
		name: "lzma".into(),
		size: Size::new(compressed.len()),
		compress: Time::new(compress_duration),
		decompress: Time::new(decompress_duration),
		lossless,
		orig_size: Size::new(data.len() * 2),
	}
}
