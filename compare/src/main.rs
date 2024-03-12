use std::{
	io::{Cursor, Write},
	path::Path,
};

use tiff::decoder::{Decoder, DecodingResult};

use crate::{
	compress::{hcomp, webp, zstd},
	output::{Compression, Size},
};

mod compress;
mod output;

fn main() {
	let mut size = 0;
	let mut disk_size = 0;
	let mut hcomp_ = Compression::named("hcomp");
	// let mut webp_ = Compression::named("webp");
	// let mut zstd_ = Compression::named("zstd");

	let count = std::env::args().skip(1).count();
	for (i, path) in std::env::args().skip(1).enumerate() {
		let path = Path::new(&path);

		let size_on_disk = path.metadata().unwrap().len() as usize;
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

		size += data.len() * 2;
		disk_size += size_on_disk;

		print!("\r{} / {}: hcomp", i + 1, count);
		std::io::stdout().flush().unwrap();
		hcomp_ += hcomp(&data, width, height);

		// print!("\r{} / {}: hcomp (delta 2x)", i + 1, count);
		// std::io::stdout().flush().unwrap();
		// hcomp_delta_2 += hcomp_delta(&data, width, height, 2);
		//
		// print!("\r{} / {}: hcomp (delta 4x)", i + 1, count);
		// std::io::stdout().flush().unwrap();
		// hcomp_delta_4 += hcomp_delta(&data, width, height, 4);
		//
		// print!("\r{} / {}: webp", i + 1, count);
		// std::io::stdout().flush().unwrap();
		// webp_ += webp(&data, width, height);
		//
		// print!("\r{} / {}: zstd", i + 1, count);
		// std::io::stdout().flush().unwrap();
		// zstd_ += zstd(&data);
	}

	println!("\rraw size: {}", Size::new(size));
	println!("size on disk: {}\n", Size::new(disk_size));
	println!("{}", hcomp_);
	// println!("{}\n{}", webp_, webp_.relative_to(&hcomp_));
	// println!("{}\n{}", zstd_, zstd_.relative_to(&hcomp_));
}

