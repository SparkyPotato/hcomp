use std::io::{self, Write};

use zstd::Encoder;

use crate::{
	byte_compress::byte_compress,
	palette::transform_palette,
	prediction::transform_prediction,
	stream::generate_stream,
	Heightmap,
};

/// Encode a heightmap. `compression_level` should be between -7 and 22, inclusive.
pub fn encode(heightmap: Heightmap, compression_level: i8, output: &mut impl Write) -> Result<(), io::Error> {
	assert_eq!(
		heightmap.data.len(),
		heightmap.width as usize * heightmap.height as usize,
		"heightmap data length must be equal to width * height"
	);

	let predicted = transform_prediction(heightmap)?;
	let paletted = transform_palette(predicted);
	let byte_compressed = byte_compress(paletted);
	let stream = generate_stream(byte_compressed);
	compress(&stream, compression_level, output)
}

fn compress(data: &[u8], compression_level: i8, output: &mut impl Write) -> Result<(), io::Error> {
	let mut encoder = Encoder::new(output, compression_level as _)?;
	encoder.set_pledged_src_size(Some(data.len() as u64))?;
	encoder.include_magicbytes(false)?;
	encoder.include_checksum(false)?;
	encoder.long_distance_matching(true)?;
	encoder.include_dictid(false)?;
	encoder.include_contentsize(false)?;
	encoder.window_log(24)?;

	encoder.write_all(&data)?;
	encoder.finish()?;

	Ok(())
}