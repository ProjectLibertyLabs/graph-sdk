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
mod errors;
pub use errors::*;

#[cfg(test)]
mod tests;

use std::ptr::NonNull;

#[repr(C)]
#[derive(Debug, Clone, PartialEq)]
pub struct DsnpGraphFFIResult<T, E> {
	pub result: Option<NonNull<T>>,
	pub error: Option<NonNull<E>>,
}

impl<T, E> DsnpGraphFFIResult<T, E> {
	pub fn new(result: T) -> Self {
		Self { result: Some(NonNull::new(Box::into_raw(Box::new(result))).unwrap()), error: None }
	}

	pub fn new_error(error: E) -> Self {
		Self { result: None, error: Some(NonNull::new(Box::into_raw(Box::new(error))).unwrap()) }
	}

	pub fn new_mut(result: *mut T) -> Self {
		Self { result: Some(NonNull::new(result).unwrap()), error: None }
	}

	pub fn new_mut_error(error: *mut E) -> Self {
		Self { result: None, error: Some(NonNull::new(error).unwrap()) }
	}
}
