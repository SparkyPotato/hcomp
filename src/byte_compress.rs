pub fn u16_slice_to_u8_slice(slice: &[u16]) -> &[u8] {
	unsafe { std::slice::from_raw_parts(slice.as_ptr() as *const u8, slice.len() * 2) }
}

pub fn u16_slice_to_u8_slice_mut(slice: &mut [u16]) -> &mut [u8] {
	unsafe { std::slice::from_raw_parts_mut(slice.as_mut_ptr() as *mut u8, slice.len() * 2) }
}

/// Compresses a slice of u16s into a slice of u8s, in place. Returns if the slice was compressed.
pub fn byte_compress(data: &mut [u16]) -> bool {
	let max_value = data.iter().copied().max().unwrap();
	if max_value <= u8::MAX as u16 {
		let len = data.len();
		let data = u16_slice_to_u8_slice_mut(data);

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

pub fn byte_decompress(input: &[u8], out: &mut [u16]) {
	for (o, &i) in out.iter_mut().zip(input.iter()) {
		*o = i as _;
	}
}
