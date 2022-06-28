use std::{borrow::Cow, fmt::Display, io::Cursor, path::Path, time::Instant};

use hcomp::{encode::encode, Heightmap};
use libwebp_sys::{
	WebPEncode,
	WebPImageHint::WEBP_HINT_GRAPH,
	WebPInitConfig,
	WebPPicture,
	WebPPictureImportRGBA,
	WebPPictureInit,
};
use tiff::decoder::{Decoder, DecodingResult};

struct Size(isize);

impl Display for Size {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let size = self.0;
		let s = size.abs();
		if s < 1000 {
			write!(f, "{} B", size)
		} else if s < 1000 * 1000 {
			write!(f, "{:.2} KB", size as f64 / 1000.0)
		} else if s < 1000 * 1000 * 1000 {
			write!(f, "{:.2} MiB", size as f64 / 1000.0 / 1000.0)
		} else {
			write!(f, "{:.2} GiB", size as f64 / 1000.0 / 1000.0 / 1000.0)
		}
	}
}

fn main() {
	let path = std::env::args().nth(1).unwrap();
	let path = Path::new(&path);

	let size_on_disk = path.metadata().unwrap().len();
	let mut decoder = Decoder::new(Cursor::new(std::fs::read(path).unwrap())).unwrap();
	let data = decoder.read_image().unwrap();
	let (width, height) = decoder.dimensions().unwrap();
	let data = match data {
		DecodingResult::U8(data) => data.into_iter().map(|x| x as u16).collect(),
		DecodingResult::U16(data) => data,
		DecodingResult::U32(data) => data.into_iter().map(|x| x as u16).collect(),
		DecodingResult::U64(data) => data.into_iter().map(|x| x as u16).collect(),
		DecodingResult::I8(data) => data.into_iter().map(|x| x as u16).collect(),
		DecodingResult::I16(data) => data.into_iter().map(|x| (x + 500) as u16).collect(),
		DecodingResult::I32(data) => data.into_iter().map(|x| (x + 500) as u16).collect(),
		DecodingResult::I64(data) => data.into_iter().map(|x| (x + 500) as u16).collect(),
		DecodingResult::F32(data) => data.into_iter().map(|x| (x + 500.0) as u16).collect(),
		DecodingResult::F64(data) => data.into_iter().map(|x| (x + 500.0) as u16).collect(),
	};

	let start = Instant::now();
	let mut _output: Vec<u8> = Vec::new();
	let hcomp_size = encode(
		Heightmap {
			width,
			height,
			data: Cow::Borrowed(&data),
		},
		22,
		&mut Vec::new(),
	)
	.unwrap();
	let hcomp_duration = start.elapsed();

	let start = Instant::now();
	let webp_size = unsafe {
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

		temp.len()
	};
	let webp_duration = start.elapsed();

	println!("raw size: {}", Size(data.len() as isize * 2));
	println!("size on disk: {}", Size(size_on_disk as isize));

	println!("hcomp:");
	println!("  size: {}", Size(hcomp_size as _));
	println!("  duration: {:.2} ms", hcomp_duration.as_secs_f64() * 1000.0);
	println!("webp:");
	println!("  size: {}", Size(webp_size as _));
	println!("  duration: {:.2} ms", webp_duration.as_secs_f64() * 1000.0);

	println!();
	let size_diff = hcomp_size as isize - webp_size as isize;
	let duration_diff = hcomp_duration.as_secs_f64() - webp_duration.as_secs_f64();
	println!("size difference: {}", Size(size_diff));
	println!("duration difference: {:.2} ms", duration_diff * 1000.0);
	println!("size percentage: {:.2}%", (size_diff as f64 / webp_size as f64) * 100.0);
	println!(
		"speed percentage: {:.2}%",
		duration_diff / webp_duration.as_secs_f64() * 100.0
	);
}
