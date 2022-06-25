//! A library for heightmap compression.

use std::borrow::Cow;

pub mod decode;
pub mod encode;

mod byte_compress;
mod palette;
mod prediction;
mod stream;

/// A heightmap image.
///
/// `data` represents the heights of each pixel, in row-major order.
/// `data.len()` must be equal to `width * height`.
#[derive(Clone)]
pub struct Heightmap<'a> {
	pub width: u32,
	pub height: u32,
	pub data: Cow<'a, [u16]>,
}
