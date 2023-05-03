use crate::bindings::*;
use dsnp_graph_config::Environment;
use dsnp_graph_core::{
	api::api::GraphAPI,
	dsnp::{api_types::PrivacyType, dsnp_types::DsnpUserId},
};
use std::os::raw::c_int;

#[no_mangle]
pub extern "C" fn graph_state_new(environment: *const Environment) -> *mut GraphState {
	let environment = unsafe { &*environment };
	let inner = Box::new(GraphState::new(environment.clone()));
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
	graph_state.inner.contains_user_graph(&DsnpUserId::new(user_id))
}

#[no_mangle]
pub extern "C" fn graph_state_len(graph_state: &GraphState) -> usize {
	graph_state.inner.len()
}

#[no_mangle]
pub extern "C" fn graph_state_remove_user_graph(graph_state: &mut GraphState, user_id: u64) {
	graph_state.inner.remove_user_graph(&DsnpUserId::new(user_id));
}

#[no_mangle]
pub extern "C" fn graph_api_create_user_graph(
	graph_api: &dyn GraphAPI,
	user_id: u64,
	privacy: CPrivacyType,
) -> c_int {
	let privacy_type = PrivacyType::from(privacy);
	match graph_api.create_user_graph(DsnpUserId::new(user_id), privacy_type) {
		Ok(()) => 1,
		Err(_) => 0,
	}
}
