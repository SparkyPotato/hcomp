use std::io::{self, Write};

use libwebp_sys::{
	WebPEncode,
	WebPImageHint::WEBP_HINT_GRAPH,
	WebPInitConfig,
	WebPPicture,
	WebPPictureFree,
	WebPPictureImportRGBA,
	WebPPictureInit,
};

use crate::{prediction::transform_prediction, Heightmap};

/// Encode a heightmap.
pub fn encode(heightmap: Heightmap, output: &mut impl Write) -> Result<usize, io::Error> {
	assert!(
		heightmap.width > 2 && heightmap.height > 2,
		"Heightmap must be at least 3x3"
	);
	assert_eq!(
		heightmap.data.len(),
		heightmap.width as usize * heightmap.height as usize,
		"heightmap data length must be equal to width * height"
	);

	let predicted = transform_prediction(heightmap.data.into(), heightmap.width, heightmap.height)?;
	return compress_webp(&predicted, heightmap.width, heightmap.height, output);
}

fn compress_webp<T: Write>(data: &[u16], width: u32, height: u32, output: &mut T) -> Result<usize, io::Error> {
	unsafe {
		let mut config = std::mem::zeroed();
		WebPInitConfig(&mut config);
		config.lossless = 1;
		config.quality = 100.0;
		config.method = 5;
		config.image_hint = WEBP_HINT_GRAPH;
		config.exact = 1;

		let mut picture = std::mem::zeroed();
		WebPPictureInit(&mut picture);
		picture.use_argb = 1;
		picture.writer = Some(write::<T>);
		let mut c = Ctx { w: output, b: 0 };
		picture.custom_ptr = &mut c as *mut _ as _;
		picture.width = width as i32;
		picture.height = height as i32;

		c.w.write(&data[0].to_le_bytes())?;

		let mut pad = Vec::with_capacity((data.len() - 1) * 2);
		for &d in data[1..].iter() {
			pad.push(d);
			pad.push(0);
		}

		WebPPictureImportRGBA(&mut picture, pad.as_ptr() as _, width as i32 * 4);
		WebPEncode(&config, &mut picture);

		if picture.error_code as i32 != 0 {
			panic!("WebPEncode failed: {}", picture.error_code as i32)
		}

		struct Ctx<'a, T: Write> {
			w: &'a mut T,
			b: usize,
		}
		unsafe extern "C" fn write<T: Write>(data: *const u8, data_size: usize, picture: *const WebPPicture) -> i32 {
			let c = &mut *((*picture).custom_ptr as *mut Ctx<T>);
			c.w.write(std::slice::from_raw_parts(data, data_size)).unwrap();
			c.b += data_size;
			1
		}

		WebPPictureFree(&mut picture);

		Ok(c.b)
	}
}

