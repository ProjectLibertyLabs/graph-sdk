use crate::{bindings::*, utils::*};
use dsnp_graph_config::SchemaId;
use dsnp_graph_core::{
	api::api::{GraphAPI, GraphState},
	dsnp::dsnp_types::{DsnpGraphEdge, DsnpUserId},
};

#[no_mangle]
pub extern "C" fn print_hello_graph() {
	println!("Hello, Graph!");
}

// Singleton for GraphState
#[no_mangle]
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

// Intialize GraphState with capacity
#[no_mangle]
pub unsafe extern "C" fn initialize_graph_state_with_capacity(
	environment: *const Environment,
	capacity: usize,
) -> bool {
	let environment = &*environment;
	let rust_environment = environment_from_ffi(environment);
	let graph_state = GraphState::with_capacity(rust_environment, capacity);
	GRAPH_STATE = Some(graph_state);
	true
}

// Get Capacity
#[no_mangle]
pub unsafe extern "C" fn get_graph_capacity() -> usize {
	if let Some(graph_state) = GRAPH_STATE.as_mut() {
		graph_state.capacity()
	} else {
		0
	}
}

// State contains user graph
#[no_mangle]
pub unsafe extern "C" fn graph_contains_user(user_id: *const DsnpUserId) -> bool {
	if let Some(graph_state) = GRAPH_STATE.as_mut() {
		let user_id = &*user_id;
		graph_state.contains_user_graph(user_id)
	} else {
		false
	}
}

// Count of users in current graph
#[no_mangle]
pub unsafe extern "C" fn graph_users_count() -> usize {
	if let Some(graph_state) = GRAPH_STATE.as_mut() {
		graph_state.len()
	} else {
		0
	}
}

// Remove user
#[no_mangle]
pub unsafe extern "C" fn graph_remove_user(user_id: *const DsnpUserId) -> bool {
	if let Some(graph_state) = GRAPH_STATE.as_mut() {
		let user_id = &*user_id;
		graph_state.remove_user_graph(user_id);
		true
	} else {
		false
	}
}

//
#[no_mangle]
pub unsafe extern "C" fn graph_import_users_data(
	payloads: *const ImportBundle,
	payloads_len: usize,
) -> bool {
	if let Some(graph_state) = GRAPH_STATE.as_mut() {
		let payloads = std::slice::from_raw_parts(payloads, payloads_len);
		let payloads = payloads_from_ffi(&payloads);
		graph_state.import_users_data(&payloads).is_ok()
	} else {
		false
	}
}

// Graph export updates fn export_updates(&mut self) -> Result<Vec<Update>> {
#[no_mangle]
pub unsafe extern "C" fn graph_export_updates() -> *mut Update {
	if let Some(graph_state) = GRAPH_STATE.as_mut() {
		match graph_state.export_updates() {
			Ok(updates) => {
				let ffi_updates = updates_to_ffi(updates);
				let _updates_len = ffi_updates.len();
				let updates_ptr = Box::into_raw(Box::new(ffi_updates));
				return updates_ptr as *mut Update
			},
			Err(_err) => {
				// handle error?
				return std::ptr::null_mut()
			},
		}
	} else {
		return std::ptr::null_mut()
	}
}

// Graph apply actions
#[no_mangle]
pub unsafe extern "C" fn graph_apply_actions(actions: *const Action, actions_len: usize) -> bool {
	if let Some(graph_state) = GRAPH_STATE.as_mut() {
		let actions = std::slice::from_raw_parts(actions, actions_len);
		let actions = actions_from_ffi(&actions);
		graph_state.apply_actions(&actions).is_ok()
	} else {
		false
	}
}

// //	fn get_connections_for_user_graph(
// 	&self,
// 	user_id: &DsnpUserId,
// 	schema_id: &SchemaId,
// 	include_pending: bool,
// ) ->

// Graph get connections for user
#[no_mangle]
pub unsafe extern "C" fn graph_get_connections_for_user(
	user_id: *const DsnpUserId,
	schema_id: *const SchemaId,
	include_pending: bool,
) -> *mut DsnpGraphEdge {
	if let Some(graph_state) = GRAPH_STATE.as_mut() {
		let user_id = &*user_id;
		let schema_id = &*schema_id;
		let connections = graph_state
			.get_connections_for_user_graph(user_id, schema_id, include_pending)
			.unwrap();
		let _connections_len = connections.len();
		let connections_ptr = Box::into_raw(Box::new(connections));
		return connections_ptr as *mut DsnpGraphEdge
	} else {
		return std::ptr::null_mut()
	}
}

// Get connections without keys
#[no_mangle]
pub unsafe extern "C" fn graph_get_connections_without_keys() -> *mut DsnpUserId {
	if let Some(graph_state) = GRAPH_STATE.as_mut() {
		let connections = graph_state.get_connections_without_keys().unwrap();
		let _connections_len = connections.len();
		let connections_ptr = Box::into_raw(Box::new(connections));
		return connections_ptr as *mut DsnpUserId
	} else {
		return std::ptr::null_mut()
	}
}

// Get one sided private friendship connections
#[no_mangle]
pub unsafe extern "C" fn graph_get_one_sided_private_friendship_connections(
	user_id: *const DsnpUserId,
) -> *mut DsnpGraphEdge {
	if let Some(graph_state) = GRAPH_STATE.as_mut() {
		let user_id = &*user_id;
		let connections =
			graph_state.get_one_sided_private_friendship_connections(user_id).unwrap();
		let _connections_len = connections.len();
		let connections_ptr = Box::into_raw(Box::new(connections));
		return connections_ptr as *mut DsnpGraphEdge
	} else {
		return std::ptr::null_mut()
	}
}

// Free GraphState
#[no_mangle]
pub unsafe extern "C" fn free_graph_state() -> bool {
	GRAPH_STATE = None;
	true
}
