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

use dsnp_graph_config::errors::{DsnpGraphError, DsnpGraphResult};

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct GraphFFIResult<T> {
	pub result: *mut T,
	pub error: *mut DsnpGraphError,
}

impl<T> GraphFFIResult<T> {
	pub fn new(result: T) -> Self {
		let result_ptr = Box::into_raw(Box::new(result));
		Self { result: result_ptr, error: std::ptr::null_mut() }
	}

	pub fn new_error(error: DsnpGraphError) -> Self {
		Self { result: std::ptr::null_mut(), error: Box::into_raw(Box::new(error)) }
	}
}

impl<T> Drop for GraphFFIResult<T> {
	fn drop(&mut self) {
		if !self.result.is_null() {
			unsafe {
				Box::from_raw(self.result);
			}
		}
		if !self.error.is_null() {
			unsafe {
				Box::from_raw(self.error);
			}
		}
	}
}

impl<T> From<DsnpGraphResult<T>> for GraphFFIResult<T> {
	fn from(result: DsnpGraphResult<T>) -> Self {
		match result {
			Ok(result) => Self::new(result),
			Err(error) => Self::new_error(error),
		}
	}
}
