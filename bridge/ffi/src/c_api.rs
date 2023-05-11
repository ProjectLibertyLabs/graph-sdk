use crate::{bindings::*, utils::*};
use dsnp_graph_config::SchemaId;
use dsnp_graph_core::{
	api::api::{GraphAPI, GraphState},
	dsnp::dsnp_types::DsnpUserId,
};
use std::{mem::ManuallyDrop, panic, sync::Mutex};

#[no_mangle]
pub extern "C" fn print_hello_graph() {
	println!("Hello, Graph!");
}

// Collection of GraphStates
#[no_mangle]
static GRAPH_STATES: Mutex<Vec<Box<GraphState>>> = Mutex::new(Vec::new());

// Initialize GraphState
#[no_mangle]
pub unsafe extern "C" fn initialize_graph_state(
	environment: *const Environment,
) -> *mut GraphState {
	// use panic_unwind
	let result = panic::catch_unwind(|| {
		let environment = &*environment;
		let rust_environment = environment_from_ffi(environment);
		let graph_state = Box::new(GraphState::new(rust_environment));
		let graph_state_ptr = Box::into_raw(graph_state);

		// Add state pointer to GRAPH_STATES vector
		GRAPH_STATES.lock().unwrap().push(unsafe { Box::from_raw(graph_state_ptr) });

		graph_state_ptr
	});
	match result {
		Ok(graph_state) => graph_state,
		Err(_) => std::ptr::null_mut(),
	}
}

// Intialize GraphState with capacity
#[no_mangle]
pub unsafe extern "C" fn initialize_graph_state_with_capacity(
	environment: *const Environment,
	capacity: usize,
) -> *mut GraphState {
	let result = panic::catch_unwind(|| {
		let environment = &*environment;
		let rust_environment = environment_from_ffi(environment);
		let graph_state = Box::new(GraphState::with_capacity(rust_environment, capacity));
		let graph_state_ptr = Box::into_raw(graph_state);

		// Add state pointer to GRAPH_STATES vector
		GRAPH_STATES.lock().unwrap().push(unsafe { Box::from_raw(graph_state_ptr) });

		graph_state_ptr
	});
	match result {
		Ok(graph_state) => graph_state,
		Err(_) => std::ptr::null_mut(),
	}
}

// Get Capacity
#[no_mangle]
pub unsafe extern "C" fn get_graph_capacity(graph_state: *mut GraphState) -> usize {
	if graph_state.is_null() {
		return 0
	}
	let graph_state = &mut *graph_state;
	graph_state.capacity()
}

// Get total graph states in GRAPH_STATES
#[no_mangle]
pub unsafe extern "C" fn get_graph_states_count() -> usize {
	let graph_states = GRAPH_STATES.lock().unwrap();
	graph_states.len()
}

// State contains user graph
#[no_mangle]
pub unsafe extern "C" fn graph_contains_user(
	graph_state: *mut GraphState,
	user_id: *const DsnpUserId,
) -> bool {
	if graph_state.is_null() {
		return false
	}
	let graph_state = &mut *graph_state;
	let user_id = &*user_id;
	graph_state.contains_user_graph(user_id)
}

// Count of users in current graph
#[no_mangle]
pub unsafe extern "C" fn graph_users_count(graph_state: *mut GraphState) -> usize {
	if graph_state.is_null() {
		return 0
	}
	let graph_state = &mut *graph_state;
	graph_state.len()
}

// Remove user
#[no_mangle]
pub unsafe extern "C" fn graph_remove_user(
	graph_state: *mut GraphState,
	user_id: *const DsnpUserId,
) -> bool {
	if graph_state.is_null() {
		return false
	}
	let graph_state = &mut *graph_state;
	let user_id = &*user_id;
	graph_state.remove_user_graph(user_id);
	true
}

//Graph import users data
#[no_mangle]
pub unsafe extern "C" fn graph_import_users_data(
	graph_state: *mut GraphState,
	payloads: *const ImportBundle,
	payloads_len: usize,
) -> bool {
	if graph_state.is_null() {
		return false
	}
	let graph_state = &mut *graph_state;
	let payloads = std::slice::from_raw_parts(payloads, payloads_len);
	let payloads = payloads_from_ffi(&payloads);
	graph_state.import_users_data(&payloads).is_ok()
}

// Graph export updates
#[no_mangle]
pub unsafe extern "C" fn graph_export_updates(graph_state: *mut GraphState) -> GraphUpdates {
	if graph_state.is_null() {
		return GraphUpdates { updates: std::ptr::null_mut(), updates_len: 0 }
	}
	let graph_state = &mut *graph_state;
	let updates = graph_state.export_updates().unwrap_or_default();
	let ffi_updates = updates_to_ffi(updates);
	let updates_len = ffi_updates.len();
	let updates_ptr = ManuallyDrop::new(ffi_updates).as_mut_ptr();
	GraphUpdates { updates: updates_ptr, updates_len }
}

// Graph apply actions
#[no_mangle]
pub unsafe extern "C" fn graph_apply_actions(
	graph_state: *mut GraphState,
	actions: *const Action,
	actions_len: usize,
) -> bool {
	if graph_state.is_null() {
		return false
	}
	let graph_state = &mut *graph_state;
	let actions = std::slice::from_raw_parts(actions, actions_len);
	let actions = actions_from_ffi(&actions);
	graph_state.apply_actions(&actions).is_ok()
}

// Graph get connections for user
#[no_mangle]
pub unsafe extern "C" fn graph_get_connections_for_user(
	graph_state: *mut GraphState,
	user_id: *const DsnpUserId,
	schema_id: *const SchemaId,
	include_pending: bool,
) -> GraphConnections {
	if graph_state.is_null() {
		return GraphConnections { connections: std::ptr::null_mut(), connections_len: 0 }
	}
	let graph_state = &mut *graph_state;
	let user_id = &*user_id;
	let schema_id = &*schema_id;
	let connections = graph_state
		.get_connections_for_user_graph(user_id, schema_id, include_pending)
		.unwrap_or_default();
	let connections_len = connections.len();
	let connections_ptr = ManuallyDrop::new(connections).as_mut_ptr();
	GraphConnections { connections: connections_ptr, connections_len }
}

// Get connections without keys
#[no_mangle]
pub unsafe extern "C" fn graph_get_connections_without_keys(
	graph_state: *mut GraphState,
) -> GraphConnectionsWithoutKeys {
	if graph_state.is_null() {
		return GraphConnectionsWithoutKeys { connections: std::ptr::null_mut(), connections_len: 0 }
	}
	let graph_state = &mut *graph_state;
	let connections = graph_state.get_connections_without_keys().unwrap_or_default();
	let connections_len = connections.len();
	let connections_ptr = ManuallyDrop::new(connections).as_mut_ptr();
	GraphConnectionsWithoutKeys { connections: connections_ptr, connections_len }
}

// Get one sided private friendship connections
#[no_mangle]
pub unsafe extern "C" fn graph_get_one_sided_private_friendship_connections(
	graph_state: *mut GraphState,
	user_id: *const DsnpUserId,
) -> GraphConnections {
	if graph_state.is_null() {
		return GraphConnections { connections: std::ptr::null_mut(), connections_len: 0 }
	}
	let graph_state = &mut *graph_state;
	let user_id = &*user_id;
	let connections = graph_state
		.get_one_sided_private_friendship_connections(user_id)
		.unwrap_or_default();
	let connections_len = connections.len();
	let connections_ptr = ManuallyDrop::new(connections).as_mut_ptr();
	GraphConnections { connections: connections_ptr, connections_len }
}

#[no_mangle]
pub extern "C" fn free_graph_state(graph_state: *mut GraphState) {
	let mut graph_states = GRAPH_STATES.lock().unwrap();
	graph_states.retain(|x| !std::ptr::eq(x.as_ref(), unsafe { &*graph_state }));
}

#[no_mangle]
pub extern "C" fn free_graph_states() {
	let mut graph_states = GRAPH_STATES.lock().unwrap();
	graph_states.clear();
}

// Free GraphUpdates
#[no_mangle]
pub unsafe extern "C" fn free_graph_updates(graph_updates: *mut GraphUpdates) {
	let _ = Box::from_raw(graph_updates);
}

// Free GraphConnections
#[no_mangle]
pub unsafe extern "C" fn free_graph_connections(graph_connections: *mut GraphConnections) {
	let _ = Box::from_raw(graph_connections);
}

// Free GraphConnectionsWithoutKeys
#[no_mangle]
pub unsafe extern "C" fn free_graph_connections_without_keys(
	graph_connections: *mut GraphConnectionsWithoutKeys,
) {
	let _ = Box::from_raw(graph_connections);
}
