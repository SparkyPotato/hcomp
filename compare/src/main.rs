use std::{io::Cursor, path::Path};

use tiff::decoder::{Decoder, DecodingResult};

use crate::{
	compress::{hcomp, webp, xz2, zstd},
	output::Size,
};

mod compress;
mod output;

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

	println!("raw size: {}", Size::new(data.len() * 2));
	println!("size on disk: {}\n", Size::new(size_on_disk as _));

	let hcomp = hcomp(&data, width, height);
	println!("{}", hcomp);

	let webp = webp(&data, width, height);
	println!("{}\n{}", webp, webp.relative_to(&hcomp));

	let zstd = zstd(&data);
	println!("{}\n{}", zstd, zstd.relative_to(&hcomp));

	let xz2 = xz2(&data);
	println!("{}\n{}", xz2, xz2.relative_to(&hcomp));
}
