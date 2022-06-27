use std::io::{self, IoSlice, Write};

use zstd::Encoder;

use crate::{
	byte_compress::{byte_compress, ByteCompressResult},
	palette::{transform_palette, PaletteResult},
	prediction::transform_prediction,
	stream::generate_stream,
	Heightmap,
};

/// Encode a heightmap. `compression_level` should be between -7 and 22, inclusive.
pub fn encode(heightmap: Heightmap, compression_level: i8, output: &mut impl Write) -> Result<usize, io::Error> {
	assert_eq!(
		heightmap.data.len(),
		heightmap.width as usize * heightmap.height as usize,
		"heightmap data length must be equal to width * height"
	);

	let predicted = transform_prediction(heightmap)?;
	// let paletted = transform_palette(predicted);
	// let byte_compressed = byte_compress(paletted);
	let stream = generate_stream(ByteCompressResult::Uncompressed(PaletteResult::Unpalleted(predicted)));
	compress(&stream, compression_level, output)
}

fn compress(data: &[u8], compression_level: i8, output: &mut impl Write) -> Result<usize, io::Error> {
	pub struct WriteWrapper<'a, T> {
		inner: &'a mut T,
		bytes_written: usize,
	}

	impl<'a, T: Write> Write for WriteWrapper<'a, T> {
		fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
			let res = self.inner.write(buf)?;
			self.bytes_written += res;
			Ok(res)
		}

		fn write_vectored(&mut self, bufs: &[IoSlice<'_>]) -> io::Result<usize> {
			let res = self.inner.write_vectored(bufs)?;
			self.bytes_written += res;
			Ok(res)
		}

		fn flush(&mut self) -> io::Result<()> { self.inner.flush() }
	}

	let mut encoder = Encoder::new(
		WriteWrapper {
			inner: output,
			bytes_written: 0,
		},
		compression_level as _,
	)?;
	encoder.set_pledged_src_size(Some(data.len() as u64))?;
	encoder.include_magicbytes(false)?;
	encoder.include_checksum(false)?;
	encoder.long_distance_matching(true)?;
	encoder.include_dictid(false)?;
	encoder.include_contentsize(false)?;
	encoder.window_log(24)?;

	encoder.write_all(&data)?;
	let writer = encoder.finish()?;

	Ok(writer.bytes_written)
}
