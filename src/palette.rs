use std::collections::HashMap;

use crate::byte_compress::u16_slice_to_u8_slice;

pub fn transform_palette(data: &mut [u8]) -> Option<usize> {
	// Paletting is not worth it.
	if data.len() <= 512 {
		return None;
	}

	let mut map = HashMap::with_capacity(256);
	for value in data.chunks_exact(2) {
		let value = u16::from_ne_bytes(value.try_into().unwrap());
		if value != 0 {
			map.insert(value, 0);
			if map.len() > 255 {
				return None;
			}
		}
	}

	let data_offset = 1 + map.len() * 2;
	let data_len = data.len() / 2;
	// Don't have enough space to fit the palette.
	if data_offset > data_len {
		return None;
	}

	let mut sorted: Vec<u16> = map.iter().map(|(&x, _)| x).collect();
	sorted.sort_unstable();

	map.insert(0, 0);
	if let Some(index) = sorted.get(0) {
		*map.get_mut(index).unwrap() = 1;
	}
	// Delta compress palette.
	for i in (1..sorted.len()).rev() {
		*map.get_mut(&sorted[i]).unwrap() = (i + 1) as u8;
		sorted[i] -= sorted[i - 1];
	}

	let palette = sorted;

	for i in (0..data_len).rev() {
		let value = u16::from_ne_bytes(data[2 * i..2 * i + 2].try_into().unwrap());
		data[data_len + i] = map[&value];
	}

	data[0] = palette.len() as _;
	data[1..data_offset].copy_from_slice(u16_slice_to_u8_slice(&palette));
	unsafe {
		std::ptr::copy(data[data_len..].as_ptr(), data[data_offset..].as_ptr() as _, data_len);
	}

	Some(data_offset + data_len)
}

pub fn decode_palette(input: &mut [u8], out: &mut [u8]) {
	let len = input[0] as usize;
	let palette = &mut input[1..1 + len * 2];

	for i in 1..len {
		let prev = u16::from_le_bytes(palette[(i - 1) * 2..i * 2].try_into().unwrap());
		let curr = u16::from_le_bytes(palette[i * 2..(i + 1) * 2].try_into().unwrap());
		palette[i * 2..(i + 1) * 2].copy_from_slice(&(prev + curr).to_le_bytes());
	}

	let palette = &input[1..1 + len * 2];

	let data = &input[1 + len * 2..];
	for (i, &h) in data.iter().enumerate() {
		let h = h as usize;
		out[i * 2..(i + 1) * 2].copy_from_slice(if h != 0 { &palette[(h - 1) * 2..h * 2] } else { &[0, 0] });
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn flat_palette() {
		let mut data = vec![0; 100 * 100 * 2];
		for (i, v) in data.chunks_exact_mut(2).take(10).enumerate() {
			let value = (i as u16).to_le_bytes();
			v.copy_from_slice(&value);
		}
		let orig = data.clone();

		let len = transform_palette(&mut data).unwrap();
		assert_eq!(data[0], 9);
		assert_eq!(data[1..3], [1, 0]);
		assert_eq!(data[3..5], [1, 0]);
		assert_eq!(data[5..7], [1, 0]);
		assert_eq!(data[7..9], [1, 0]);
		assert_eq!(data[9..11], [1, 0]);
		assert_eq!(data[11..13], [1, 0]);
		assert_eq!(data[13..15], [1, 0]);
		assert_eq!(data[15..17], [1, 0]);
		assert_eq!(data[17..19], [1, 0]);
		assert_eq!(data[19], 0);
		assert_eq!(data[20], 1);
		assert_eq!(data[21], 2);
		assert_eq!(data[22], 3);
		assert_eq!(data[23], 4);
		assert_eq!(data[24], 5);
		assert_eq!(data[25], 6);
		assert_eq!(data[26], 7);
		assert_eq!(data[27], 8);
		assert_eq!(data[28], 9);
		assert_eq!(data[29], 0);
		assert_eq!(len, 1 + 9 * 2 + 100 * 100);

		let mut out = vec![0; 100 * 100 * 2];
		decode_palette(&mut data[..len], &mut out);
		assert_eq!(orig, out);
	}
}
