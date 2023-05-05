use crate::{bindings::*, utils::*};
use dsnp_graph_core::api::api::GraphState as GraphStateRust;
use libc::c_void;

#[no_mangle]
pub extern "C" fn print_hello_graph() {
	println!("Hello, Graph!");
}

// Define a C-compatible representation of GraphState
#[repr(C)]
pub struct GraphState {
	inner: *mut c_void,
}

// Acquire a reference to the Rust GraphState
#[no_mangle]
pub unsafe extern "C" fn graph_state_new(environment: *const Environment) -> *mut GraphState {
	let environment = &*environment;
	let rust_environment = environment_from_ffi(environment);
	let graph_state = GraphStateRust::new(rust_environment);
	let c_graph_state = GraphState { inner: Box::into_raw(Box::new(graph_state)) as *mut c_void };
	Box::into_raw(Box::new(c_graph_state))
}

// Free the Rust GraphState
#[no_mangle]
pub unsafe extern "C" fn graph_state_free(graph_state: *mut GraphState) {
	if graph_state.is_null() {
		return
	}
	let graph_state = Box::from_raw(graph_state);
	let _ = Box::from_raw(graph_state.inner as *mut GraphStateRust);
}
