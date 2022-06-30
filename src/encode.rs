use std::io::{self, IoSlice, Write};

use zstd::Encoder;

use crate::{
	byte_compress::{byte_compress, u16_slice_to_u8_slice, u16_slice_to_u8_slice_mut},
	palette::transform_palette,
	prediction::transform_prediction,
	Heightmap,
};

/// Encode a heightmap. `compression_level` should be between -7 and 22, inclusive.
pub fn encode(heightmap: Heightmap, compression_level: i8, output: &mut impl Write) -> Result<usize, io::Error> {
	assert!(
		heightmap.width > 2 && heightmap.height > 2,
		"Heightmap must be at least 3x3"
	);
	assert_eq!(
		heightmap.data.len(),
		heightmap.width as usize * heightmap.height as usize,
		"heightmap data length must be equal to width * height"
	);

	let mut predicted = transform_prediction(heightmap.data.into(), heightmap.width, heightmap.height)?;
	let delta_count = predicted.len() - 2;
	let bytes = u16_slice_to_u8_slice_mut(&mut predicted);
	let deltas = &mut bytes[4..];

	let len = if byte_compress(deltas) {
		delta_count
	} else {
		match transform_palette(deltas) {
			Some(len) => {
				let data_offset = len - delta_count;
				if byte_compress(&mut deltas[1..data_offset]) {
					unsafe {
						let palette_count = (data_offset - 1) / 2;
						std::ptr::copy(
							deltas[data_offset..].as_ptr(),
							deltas[1 + palette_count..].as_mut_ptr(),
							delta_count,
						);
						1 + palette_count + delta_count
					}
				} else {
					len
				}
			},
			None => deltas.len(),
		}
	};
	compress(&bytes[..4 + len], compression_level, output)
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
