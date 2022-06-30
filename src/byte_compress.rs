pub fn u16_slice_to_u8_slice(slice: &[u16]) -> &[u8] {
	unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, slice.len() * 2) }
}

pub fn u16_slice_to_u8_slice_mut(slice: &mut [u16]) -> &mut [u8] {
	unsafe { std::slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut u8, slice.len() * 2) }
}

/// Compresses a slice of u16s into a slice of u8s, in place. Returns if the slice was compressed.
pub fn byte_compress(data: &mut [u8]) -> bool {
	let max_value = data
		.chunks_exact(2)
		.map(|x| u16::from_ne_bytes(x.try_into().unwrap()))
		.max()
		.unwrap();
	if max_value <= u8::MAX as u16 {
		let len = data.len() / 2;

		#[cfg(target_endian = "little")]
		for i in 1..len {
			data[i] = data[i * 2];
		}

		#[cfg(target_endian = "big")]
		for i in 0..len {
			data[i] = data[i * 2 + 1];
		}

		true
	} else {
		false
	}
}

pub fn byte_decompress(input: &[u8], out: &mut [u8]) {
	for (o, &i) in out.chunks_exact_mut(2).zip(input.iter()) {
		#[cfg(target_endian = "little")]
		{
			o[0] = i;
		}

		#[cfg(target_endian = "big")]
		{
			o[1] = i;
		}
	}
}
