use std::{mem, path::Path, slice};
use libc::{c_char, c_void};
use crate::{ffi, Version};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Header {
	pub version: Version,

	pub image_width: u16,
	pub image_height: u16,
	pub bit_depth: u8,
	pub image_count: u64,

	pub labels: Vec<String>,
}

impl Header {
	#[inline]
	pub fn image_size(&self) -> usize {
		self.image_width as usize *
		self.image_height as usize *
		self.bit_depth as usize / 8
	}

	pub fn read_from_path<P: AsRef<Path>>(path: P) -> crate::Result<Self> {
		let path = path
			.as_ref()
			.to_str()
			.unwrap();

		let path_cstring = std::ffi::CString::new(path).unwrap();
		let header_ptr = unsafe { ffi::JDX_AllocHeader() };
		let read_error = unsafe { ffi::JDX_ReadHeaderFromPath(header_ptr, path_cstring.as_ptr()) };

		if let Some(error) = crate::Error::new_with_path(read_error, path) {
			return Err(error);
		}

		return Ok(header_ptr.into());
	}
}

impl From<*mut ffi::JDXHeader> for Header {
	fn from(header_ptr: *mut ffi::JDXHeader) -> Self {
		unsafe {
			let header = *header_ptr;

			let labels = slice::from_raw_parts(header.labels, header.label_count as usize)
				.iter()
				.map(|&label| std::ffi::CStr::from_ptr(label).to_string_lossy().into_owned())
				.collect();

			ffi::JDX_FreeHeader(header_ptr);

			return Self {
				version: header.version.into(),
				image_width: header.image_width,
				image_height: header.image_height,
				bit_depth: header.bit_depth,
				image_count: header.image_count,
				labels: labels,
			};
		}
	}
}

impl From<&Header> for *mut ffi::JDXHeader {
	fn from(header: &Header) -> *mut ffi::JDXHeader {
		unsafe {
			let header_ptr = ffi::JDX_AllocHeader();

			let labels = header.labels
				.iter()
				.map(|label| { // TODO: Consider doing this directly with malloc to avoid extra allocation
					let label_cstr = std::ffi::CString::new(label.clone()).unwrap();
					return libc::strdup(label_cstr.as_ptr());
				})
				.collect::<Vec<*mut c_char>>();

			let labels_ptr = crate::memdup(
				labels.as_ptr() as *const c_void,
				mem::size_of_val(&labels as &[*mut c_char]
			)) as *mut *mut c_char;

			*header_ptr = ffi::JDXHeader {
				version: header.version.into(),
				image_count: header.image_count,
				image_width: header.image_width,
				image_height: header.image_height,
				bit_depth: header.bit_depth,
				labels: labels_ptr,
				label_count: header.labels.len() as u16, // TODO: Consider checking this cast
			};

			return header_ptr;
		}
	}
}
