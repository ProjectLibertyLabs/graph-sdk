//! # Graph SDK FFI Module
//!
//! This module provides a C-compatible FFI (Foreign Function Interface) layer for the Graph SDK.
//! This FFI layer allows you to call Graph SDK functions from C, C++, or other programming languages
//! that support FFI.
//!
//! ## Usage
//!
//! To use the FFI layer, you need to build the dynamic library (e.g., .so, .dylib, or .dll) by adding
//! the appropriate configuration to `Cargo.toml`. Then, include the generated dynamic library and
//! the C header file `dsnp_graph_sdk_ffi.h` in your C, C++, or other FFI-compatible projects.

mod c_api;
pub use c_api::*;
mod bindings;
pub use bindings::*;
mod utils;
pub use utils::*;

#[cfg(test)]
mod tests;

use std::ptr::NonNull;

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct DsnpGraphFFIResult<T, E> {
	pub result: NonNull<T>,
	pub error: NonNull<E>,
}

impl<T, E> DsnpGraphFFIResult<T, E> {
	pub fn new(result: T) -> Self {
		Self {
			result: NonNull::new(Box::into_raw(Box::new(result))).unwrap(),
			error: NonNull::dangling(),
		}
	}

	pub fn new_error(error: E) -> Self {
		Self {
			result: NonNull::dangling(),
			error: NonNull::new(Box::into_raw(Box::new(error))).unwrap(),
		}
	}

	pub fn new_mut(result: *mut T) -> Self {
		Self { result: NonNull::new(result).unwrap(), error: NonNull::dangling() }
	}

	pub fn new_mut_error(error: *mut E) -> Self {
		Self { result: NonNull::dangling(), error: NonNull::new(error).unwrap() }
	}
}

impl<T, E> Drop for DsnpGraphFFIResult<T, E> {
	fn drop(&mut self) {
		if !self.result.as_ptr().is_null() {
			unsafe { Box::from_raw(self.result.as_ptr()) };
		}
		if !self.error.as_ptr().is_null() {
			unsafe { Box::from_raw(self.error.as_ptr()) };
		}
	}
}
