use std::{fmt::Display, ops::AddAssign, time::Duration};

pub struct Size(usize);

impl Size {
	pub fn new(bytes: usize) -> Self { Size(bytes) }
}

impl Display for Size {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let size = self.0;
		if size < 1000 {
			write!(f, "{} B", size)
		} else if size < 1000 * 1000 {
			write!(f, "{:.2} KB", size as f64 / 1000.0)
		} else if size < 1000 * 1000 * 1000 {
			write!(f, "{:.2} MiB", size as f64 / 1000.0 / 1000.0)
		} else {
			write!(f, "{:.2} GiB", size as f64 / 1000.0 / 1000.0 / 1000.0)
		}
	}
}

pub struct Time(f32);

impl Time {
	pub fn new(duration: Duration) -> Self { Time(duration.as_secs_f32()) }
}

impl Display for Time {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		let secs = self.0;
		if secs < 1.0 {
			write!(f, "{:.2} ms", secs * 1000.0)
		} else {
			write!(f, "{:.2} s", secs)
		}
	}
}

pub struct Compression {
	pub name: String,
	pub size: Size,
	pub orig_size: Size,
	pub compress: Time,
	pub decompress: Time,
	pub lossless: bool,
}

impl Compression {
	pub fn relative_to<'a>(&'a self, other: &'a Compression) -> RelativeCompression<'a> {
		RelativeCompression {
			first: self,
			second: other,
		}
	}

	pub fn named(name: &str) -> Self {
		Self {
			name: name.into(),
			size: Size(0),
			orig_size: Size(0),
			compress: Time(0.0),
			decompress: Time(0.0),
			lossless: true,
		}
	}
}

impl AddAssign for Compression {
	fn add_assign(&mut self, rhs: Self) {
		self.size.0 += rhs.size.0;
		self.orig_size.0 += rhs.orig_size.0;
		self.compress.0 += rhs.compress.0;
		self.decompress.0 += rhs.decompress.0;
		self.lossless &= rhs.lossless;
	}
}

impl Display for Compression {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		writeln!(f, "{}:", self.name)?;
		writeln!(f, "  size: {}", self.size)?;
		writeln!(f, "  compress: {}", self.compress)?;
		writeln!(f, "  decompress: {}", self.decompress)?;
		writeln!(f, "  lossless: {}", self.lossless)?;
		writeln!(
			f,
			"  compression ratio: {:.2}%",
			self.size.0 as f32 / self.orig_size.0 as f32 * 100.0
		)
	}
}

pub struct RelativeCompression<'a> {
	first: &'a Compression,
	second: &'a Compression,
}

impl Display for RelativeCompression<'_> {
	fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
		writeln!(
			f,
			"  relative size: {:.2}%",
			self.first.size.0 as f32 / self.second.size.0 as f32 * 100.0
		)?;
		writeln!(
			f,
			"  relative compress: {:.2}%",
			self.first.compress.0 / self.second.compress.0 * 100.0
		)?;
		writeln!(
			f,
			"  relative decompress: {:.2}%",
			self.first.decompress.0 / self.second.decompress.0 * 100.0
		)
	}
}

