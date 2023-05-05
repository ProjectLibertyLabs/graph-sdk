use crate::{bindings::*, utils::*};
use dsnp_graph_core::api::api::GraphState;

#[no_mangle]
pub extern "C" fn print_hello_graph() {
	println!("Hello, Graph!");
}

// Singleton for GraphState
static mut GRAPH_STATE: Option<GraphState> = None;

// Intialize GraphState
#[no_mangle]
pub unsafe extern "C" fn initialize_graph_state(environment: *const Environment) -> bool {
	let environment = &*environment;
	let rust_environment = environment_from_ffi(environment);
	let graph_state = GraphState::new(rust_environment);
	GRAPH_STATE = Some(graph_state);
	true
}

// Free GraphState
#[no_mangle]
pub unsafe extern "C" fn free_graph_state() -> bool {
	GRAPH_STATE = None;
	true
}
