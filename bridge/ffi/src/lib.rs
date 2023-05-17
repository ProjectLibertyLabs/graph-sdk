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
