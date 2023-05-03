use crate::bindings::*;
use dsnp_graph_config::Environment;
use dsnp_graph_core::api::api::{GraphAPI, GraphState as InnerGraphState};

#[no_mangle]
pub extern "C" fn graph_state_new(environment: *const Environment) -> *mut GraphState {
	let environment = unsafe { &*environment };
	let inner = Box::new(InnerGraphState::new(environment.clone()));
	let graph_state = GraphState { inner };
	Box::into_raw(Box::new(graph_state))
}

#[no_mangle]
pub extern "C" fn graph_state_free(graph_state: *mut GraphState) {
	if !graph_state.is_null() {
		unsafe { Box::from_raw(graph_state) };
	}
}

#[no_mangle]
pub extern "C" fn graph_state_contains_user_graph(graph_state: &GraphState, user_id: u64) -> bool {
	graph_state.inner.contains_user_graph(&user_id)
}

#[no_mangle]
pub extern "C" fn graph_state_len(graph_state: &GraphState) -> usize {
	graph_state.inner.len()
}

#[no_mangle]
pub extern "C" fn graph_state_remove_user_graph(graph_state: &mut GraphState, user_id: u64) {
	graph_state.inner.remove_user_graph(&user_id)
}
// TODO - add more ffi functions here
