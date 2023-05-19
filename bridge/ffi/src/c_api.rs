use crate::{bindings::*, utils::*, GraphFFIResult};
use dsnp_graph_config::{errors::DsnpGraphError, SchemaId};
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
static GRAPH_STATES: Mutex<Vec<GraphState>> = Mutex::new(Vec::new());

#[no_mangle]
pub unsafe extern "C" fn initialize_graph_state(
	environment: *const Environment,
) -> GraphFFIResult<GraphState> {
	let result = panic::catch_unwind(|| {
		let environment = &*environment;
		let rust_environment = environment_from_ffi(environment);
		let graph_state = GraphState::new(rust_environment);
		let mut graph_states = GRAPH_STATES.lock().unwrap();
		graph_states.push(unsafe { std::ptr::read(&graph_state) });
		graph_state
	});
	match result {
		Ok(graph_state) => GraphFFIResult::new(graph_state),
		Err(error) => GraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to initialize graph state: {:?}",
			error
		))),
	}
}

// Intialize GraphState with capacity
#[no_mangle]
pub unsafe extern "C" fn initialize_graph_state_with_capacity(
	environment: *const Environment,
	capacity: usize,
) -> GraphFFIResult<GraphState> {
	let result = panic::catch_unwind(|| {
		let environment = &*environment;
		let rust_environment = environment_from_ffi(environment);
		let graph_state = GraphState::with_capacity(rust_environment, capacity);
		let mut graph_states = GRAPH_STATES.lock().unwrap();
		graph_states.push(unsafe { std::ptr::read(&graph_state) });
		graph_state
	});
	match result {
		Ok(graph_state) => GraphFFIResult::new(graph_state),
		Err(error) => GraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to initialize graph state: {:?}",
			error
		))),
	}
}

// Get Capacity
#[no_mangle]
pub unsafe extern "C" fn get_graph_capacity(graph_state: *mut GraphState) -> GraphFFIResult<usize> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return 0
		}
		let graph_state = &mut *graph_state;
		graph_state.capacity()
	});
	match result {
		Ok(capacity) => GraphFFIResult::new(capacity),
		Err(error) => GraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to get graph capacity: {:?}",
			error
		))),
	}
}

// Get total graph states in GRAPH_STATES
#[no_mangle]
pub unsafe extern "C" fn get_graph_states_count() -> GraphFFIResult<usize> {
	let result = panic::catch_unwind(|| {
		let graph_states = GRAPH_STATES.lock().unwrap();
		graph_states.len()
	});
	match result {
		Ok(count) => GraphFFIResult::new(count),
		Err(error) => GraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to get graph states count: {:?}",
			error
		))),
	}
}

// State contains user graph
#[no_mangle]
pub unsafe extern "C" fn graph_contains_user(
	graph_state: *mut GraphState,
	user_id: *const DsnpUserId,
) -> GraphFFIResult<bool> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return false
		}
		let graph_state = &mut *graph_state;
		let user_id = &*user_id;
		graph_state.contains_user_graph(user_id)
	});
	match result {
		Ok(contains_user) => GraphFFIResult::new(contains_user),
		Err(error) => GraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to check if graph contains user: {:?}",
			error
		))),
	}
}

// Count of users in current graph
#[no_mangle]
pub unsafe extern "C" fn graph_users_count(graph_state: *mut GraphState) -> GraphFFIResult<usize> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return 0
		}
		let graph_state = &mut *graph_state;
		graph_state.len()
	});
	match result {
		Ok(count) => GraphFFIResult::new(count),
		Err(error) => GraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to get graph users count: {:?}",
			error
		))),
	}
}

// Remove user
#[no_mangle]
pub unsafe extern "C" fn graph_remove_user(
	graph_state: *mut GraphState,
	user_id: *const DsnpUserId,
) -> GraphFFIResult<bool> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return false
		}
		let graph_state = &mut *graph_state;
		let user_id = &*user_id;
		graph_state.remove_user_graph(user_id);
		true
	});
	match result {
		Ok(removed) => GraphFFIResult::new(removed),
		Err(error) => GraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to remove user from graph: {:?}",
			error
		))),
	}
}

// Graph import users data
#[no_mangle]
pub unsafe extern "C" fn graph_import_users_data(
	graph_state: *mut GraphState,
	payloads: *const ImportBundle,
	payloads_len: usize,
) -> GraphFFIResult<bool> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return false
		}
		let graph_state = &mut *graph_state;
		let payloads = std::slice::from_raw_parts(payloads, payloads_len);
		let payloads = payloads_from_ffi(&payloads);
		graph_state.import_users_data(&payloads).is_ok()
	});
	match result {
		Ok(imported) => GraphFFIResult::new(imported),
		Err(error) => GraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to import users data into graph: {:?}",
			error
		))),
	}
}

// Graph export updates
#[no_mangle]
pub unsafe extern "C" fn graph_export_updates(
	graph_state: *mut GraphState,
) -> GraphFFIResult<GraphUpdates> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return GraphUpdates { updates: std::ptr::null_mut(), updates_len: 0 }
		}
		let graph_state = &mut *graph_state;
		let updates = graph_state.export_updates().unwrap_or_default();
		let ffi_updates = updates_to_ffi(updates);
		let updates_len = ffi_updates.len();
		let updates_ptr = ManuallyDrop::new(ffi_updates).as_mut_ptr();
		GraphUpdates { updates: updates_ptr, updates_len }
	});
	match result {
		Ok(graph_updates) => GraphFFIResult::new(graph_updates),
		Err(error) => GraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to export updates from graph: {:?}",
			error
		))),
	}
}

// Graph apply actions
#[no_mangle]
pub unsafe extern "C" fn graph_apply_actions(
	graph_state: *mut GraphState,
	actions: *const Action,
	actions_len: usize,
) -> GraphFFIResult<bool> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return false
		}
		let graph_state = &mut *graph_state;
		let actions = std::slice::from_raw_parts(actions, actions_len);
		let actions = actions_from_ffi(&actions);
		graph_state.apply_actions(&actions).is_ok()
	});
	match result {
		Ok(applied) => GraphFFIResult::new(applied),
		Err(error) => GraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to apply actions to graph: {:?}",
			error
		))),
	}
}

// Graph get connections for user
#[no_mangle]
pub unsafe extern "C" fn graph_get_connections_for_user(
	graph_state: *mut GraphState,
	user_id: *const DsnpUserId,
	schema_id: *const SchemaId,
	include_pending: bool,
) -> GraphFFIResult<GraphConnections> {
	let result = panic::catch_unwind(|| {
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
	});
	match result {
		Ok(graph_connections) => GraphFFIResult::new(graph_connections),
		Err(error) => GraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to get connections for user from graph: {:?}",
			error
		))),
	}
}

// Get connections without keys
#[no_mangle]
pub unsafe extern "C" fn graph_get_connections_without_keys(
	graph_state: *mut GraphState,
) -> GraphFFIResult<GraphConnectionsWithoutKeys> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return GraphConnectionsWithoutKeys {
				connections: std::ptr::null_mut(),
				connections_len: 0,
			}
		}
		let graph_state = &mut *graph_state;
		let connections = graph_state.get_connections_without_keys().unwrap_or_default();
		let connections_len = connections.len();
		let connections_ptr = ManuallyDrop::new(connections).as_mut_ptr();
		GraphConnectionsWithoutKeys { connections: connections_ptr, connections_len }
	});
	match result {
		Ok(graph_connections) => GraphFFIResult::new(graph_connections),
		Err(error) => GraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to get connections without keys from graph: {:?}",
			error
		))),
	}
}

// Get one sided private friendship connections
#[no_mangle]
pub unsafe extern "C" fn graph_get_one_sided_private_friendship_connections(
	graph_state: *mut GraphState,
	user_id: *const DsnpUserId,
) -> GraphFFIResult<GraphConnections> {
	let result = panic::catch_unwind(|| {
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
	});
	match result {
		Ok(graph_connections) => GraphFFIResult::new(graph_connections),
		Err(error) => GraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to get one-sided private friendship connections from graph: {:?}",
			error
		))),
	}
}

// Get a list published and imported public keys associated with a user
#[no_mangle]
pub unsafe extern "C" fn graph_get_public_keys(
	graph_state: *mut GraphState,
	user_id: *const DsnpUserId,
) -> GraphFFIResult<DsnpPublicKeys> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return DsnpPublicKeys { keys: std::ptr::null_mut(), keys_len: 0 }
		}
		let graph_state = &mut *graph_state;
		let user_id = &*user_id;
		let keys = graph_state.get_public_keys(user_id).unwrap_or_default();
		let ffi_keys = dsnp_public_keys_to_ffi(keys);
		let keys_len = ffi_keys.len();
		let keys_ptr = ManuallyDrop::new(ffi_keys).as_mut_ptr();
		DsnpPublicKeys { keys: keys_ptr, keys_len }
	});
	match result {
		Ok(public_keys) => GraphFFIResult::new(public_keys),
		Err(error) => GraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to get public keys from graph: {:?}",
			error
		))),
	}
}

// free graph state
#[no_mangle]
pub unsafe extern "C" fn free_graph_state(graph_state: *mut GraphState) {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return
		}
		let mut graph_states = GRAPH_STATES.lock().unwrap();
		let index = graph_states.iter().position(|x| x as *const _ == graph_state).unwrap();
		graph_states.remove(index);
	});
	result.unwrap_or(())
}

// Free GraphStates
#[no_mangle]
pub extern "C" fn free_graph_states() {
	let result = panic::catch_unwind(|| {
		let mut graph_states = GRAPH_STATES.lock().unwrap();
		graph_states.clear();
	});
	result.unwrap_or(())
}

// Free GraphUpdates
#[no_mangle]
pub unsafe extern "C" fn free_graph_updates(graph_updates: *mut GraphUpdates) {
	let result = panic::catch_unwind(|| {
		let _ = Box::from_raw(graph_updates);
	});
	result.unwrap_or(())
}

// Free GraphConnections
#[no_mangle]
pub unsafe extern "C" fn free_graph_connections(graph_connections: *mut GraphConnections) {
	let result = panic::catch_unwind(|| {
		let _ = Box::from_raw(graph_connections);
	});
	result.unwrap_or(())
}

// Free GraphConnectionsWithoutKeys
#[no_mangle]
pub unsafe extern "C" fn free_graph_connections_without_keys(
	graph_connections: *mut GraphConnectionsWithoutKeys,
) {
	let result = panic::catch_unwind(|| {
		let _ = Box::from_raw(graph_connections);
	});
	result.unwrap_or(())
}

// Free DsnpPublicKeys
#[no_mangle]
pub unsafe extern "C" fn free_graph_dsnp_public_keys(public_keys: *mut DsnpPublicKeys) {
	let result = panic::catch_unwind(|| {
		let _ = Box::from_raw(public_keys);
	});
	result.unwrap_or(())
}
