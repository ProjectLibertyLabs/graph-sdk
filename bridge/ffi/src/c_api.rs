use crate::{bindings::*, utils::*, DsnpGraphFFIResult};
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
static GRAPH_STATES: Mutex<Vec<Box<GraphState>>> = Mutex::new(Vec::new());

#[no_mangle]
pub unsafe extern "C" fn initialize_graph_state(
	environment: *const Environment,
) -> DsnpGraphFFIResult<GraphState, DsnpGraphError> {
	let result = panic::catch_unwind(|| {
		let environment = &*environment;
		let rust_environment = environment_from_ffi(environment);
		let graph_state = Box::new(GraphState::new(rust_environment));
		let graph_state_ptr = Box::into_raw(graph_state);
		GRAPH_STATES.lock().unwrap().push(unsafe { Box::from_raw(graph_state_ptr) });
		DsnpGraphFFIResult::new_mut(graph_state_ptr)
	});
	result.unwrap_or_else(|error| {
		DsnpGraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to initialize graph state: {:?}",
			error
		)))
	})
}

// Intialize GraphState with capacity
#[no_mangle]
pub unsafe extern "C" fn initialize_graph_state_with_capacity(
	environment: *const Environment,
	capacity: usize,
) -> DsnpGraphFFIResult<GraphState, DsnpGraphError> {
	let result = panic::catch_unwind(|| {
		let environment = &*environment;
		let rust_environment = environment_from_ffi(environment);
		let graph_state = Box::new(GraphState::with_capacity(rust_environment, capacity));
		let graph_state_ptr = Box::into_raw(graph_state);
		GRAPH_STATES.lock().unwrap().push(unsafe { Box::from_raw(graph_state_ptr) });
		DsnpGraphFFIResult::new_mut(graph_state_ptr)
	});
	result.unwrap_or_else(|error| {
		DsnpGraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to initialize graph state with capacity: {:?}",
			error
		)))
	})
}

// Get Capacity
#[no_mangle]
pub unsafe extern "C" fn get_graph_capacity(
	graph_state: *mut GraphState,
) -> DsnpGraphFFIResult<usize, DsnpGraphError> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return DsnpGraphFFIResult::new_error(DsnpGraphError::FFIError(
				"Graph state is null".to_string(),
			))
		}
		let graph_state = &mut *graph_state;
		DsnpGraphFFIResult::new(graph_state.capacity())
	});
	result.unwrap_or_else(|error| {
		DsnpGraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to get graph capacity: {:?}",
			error
		)))
	})
}

// Get total graph states in GRAPH_STATES
#[no_mangle]
pub unsafe extern "C" fn get_graph_states_count() -> DsnpGraphFFIResult<usize, DsnpGraphError> {
	let result = panic::catch_unwind(|| {
		let graph_states = GRAPH_STATES.lock().unwrap();
		DsnpGraphFFIResult::new(graph_states.len())
	});
	result.unwrap_or_else(|error| {
		DsnpGraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to get graph states count: {:?}",
			error
		)))
	})
}

// State contains user graph
#[no_mangle]
pub unsafe extern "C" fn graph_contains_user(
	graph_state: *mut GraphState,
	user_id: *const DsnpUserId,
) -> DsnpGraphFFIResult<bool, DsnpGraphError> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return DsnpGraphFFIResult::new_error(DsnpGraphError::FFIError(
				"Graph state is null".to_string(),
			))
		}
		let graph_state = &mut *graph_state;
		let user_id = &*user_id;
		DsnpGraphFFIResult::new(graph_state.contains_user_graph(user_id))
	});
	result.unwrap_or_else(|error| {
		DsnpGraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to check graph state for user: {:?}",
			error
		)))
	})
}

// Count of users in current graph
#[no_mangle]
pub unsafe extern "C" fn graph_users_count(
	graph_state: *mut GraphState,
) -> DsnpGraphFFIResult<usize, DsnpGraphError> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return DsnpGraphFFIResult::new_error(DsnpGraphError::FFIError(
				"Graph state is null".to_string(),
			))
		}
		let graph_state = &mut *graph_state;
		DsnpGraphFFIResult::new(graph_state.len())
	});
	result.unwrap_or_else(|error| {
		DsnpGraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to get users count from graph: {:?}",
			error
		)))
	})
}

// Remove user
#[no_mangle]
pub unsafe extern "C" fn graph_remove_user(
	graph_state: *mut GraphState,
	user_id: *const DsnpUserId,
) -> DsnpGraphFFIResult<bool, DsnpGraphError> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return DsnpGraphFFIResult::new_error(DsnpGraphError::FFIError(
				"Graph state is null".to_string(),
			))
		}
		let graph_state = &mut *graph_state;
		let user_id = &*user_id;
		graph_state.remove_user_graph(user_id);
		DsnpGraphFFIResult::new(true)
	});
	result.unwrap_or_else(|error| {
		DsnpGraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to remove user from graph: {:?}",
			error
		)))
	})
}

// Graph import users data
#[no_mangle]
pub unsafe extern "C" fn graph_import_users_data(
	graph_state: *mut GraphState,
	payloads: *const ImportBundle,
	payloads_len: usize,
) -> DsnpGraphFFIResult<bool, DsnpGraphError> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return DsnpGraphFFIResult::new_error(DsnpGraphError::FFIError(
				"Graph state is null".to_string(),
			))
		}
		let graph_state = &mut *graph_state;
		let payloads = std::slice::from_raw_parts(payloads, payloads_len);
		let payloads = payloads_from_ffi(&payloads);
		let imported = graph_state.import_users_data(&payloads);
		match imported {
			Ok(_) => DsnpGraphFFIResult::new(true),
			Err(error) => DsnpGraphFFIResult::new_error(error),
		}
	});
	result.unwrap_or_else(|error| {
		DsnpGraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to import users data to graph: {:?}",
			error
		)))
	})
}

// Graph export updates
#[no_mangle]
pub unsafe extern "C" fn graph_export_updates(
	graph_state: *mut GraphState,
) -> DsnpGraphFFIResult<GraphUpdates, DsnpGraphError> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return DsnpGraphFFIResult::new_error(DsnpGraphError::FFIError(
				"Graph state is null".to_string(),
			))
		}
		let graph_state = &mut *graph_state;
		match graph_state.export_updates() {
			Ok(updates) => {
				let ffi_updates = updates_to_ffi(updates);
				let updates_len = ffi_updates.len();
				let updates_ptr = ManuallyDrop::new(ffi_updates).as_mut_ptr();
				let graph_updates = GraphUpdates { updates: updates_ptr, updates_len };
				DsnpGraphFFIResult::new(graph_updates)
			},
			Err(error) => DsnpGraphFFIResult::new_error(error),
		}
	});
	result.unwrap_or_else(|error| {
		DsnpGraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to export updates from graph: {:?}",
			error
		)))
	})
}

// Graph apply actions
#[no_mangle]
pub unsafe extern "C" fn graph_apply_actions(
	graph_state: *mut GraphState,
	actions: *const Action,
	actions_len: usize,
) -> DsnpGraphFFIResult<bool, DsnpGraphError> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return DsnpGraphFFIResult::new_error(DsnpGraphError::FFIError(
				"Graph state is null".to_string(),
			))
		}
		let graph_state = &mut *graph_state;
		let actions = std::slice::from_raw_parts(actions, actions_len);
		let actions = actions_from_ffi(&actions);
		match graph_state.apply_actions(&actions) {
			Ok(_) => DsnpGraphFFIResult::new(true),
			Err(error) => DsnpGraphFFIResult::new_error(error),
		}
	});
	result.unwrap_or_else(|error| {
		DsnpGraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to apply actions to graph: {:?}",
			error
		)))
	})
}

// Graph get connections for user
#[no_mangle]
pub unsafe extern "C" fn graph_get_connections_for_user(
	graph_state: *mut GraphState,
	user_id: *const DsnpUserId,
	schema_id: *const SchemaId,
	include_pending: bool,
) -> DsnpGraphFFIResult<GraphConnections, DsnpGraphError> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return DsnpGraphFFIResult::new_error(DsnpGraphError::FFIError(
				"Graph state is null".to_string(),
			))
		}
		let graph_state = &mut *graph_state;
		let user_id = &*user_id;
		let schema_id = &*schema_id;
		match graph_state.get_connections_for_user_graph(user_id, schema_id, include_pending) {
			Ok(connections) => {
				let connections_len = connections.len();
				let connections_ptr = ManuallyDrop::new(connections).as_mut_ptr();
				let graph_connections =
					GraphConnections { connections: connections_ptr, connections_len };
				DsnpGraphFFIResult::new(graph_connections)
			},
			Err(error) => DsnpGraphFFIResult::new_error(error),
		}
	});
	result.unwrap_or_else(|error| {
		DsnpGraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed get connections for user from graph: {:?}",
			error
		)))
	})
}

// Get connections without keys
#[no_mangle]
pub unsafe extern "C" fn graph_get_connections_without_keys(
	graph_state: *mut GraphState,
) -> DsnpGraphFFIResult<GraphConnectionsWithoutKeys, DsnpGraphError> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return DsnpGraphFFIResult::new_error(DsnpGraphError::FFIError(
				"Graph state is null".to_string(),
			))
		}
		let graph_state = &mut *graph_state;
		match graph_state.get_connections_without_keys() {
			Ok(connections) => {
				let connections_len = connections.len();
				let connections_ptr = ManuallyDrop::new(connections).as_mut_ptr();
				let graph_connections =
					GraphConnectionsWithoutKeys { connections: connections_ptr, connections_len };
				DsnpGraphFFIResult::new(graph_connections)
			},
			Err(error) => DsnpGraphFFIResult::new_error(error),
		}
	});
	result.unwrap_or_else(|error| {
		DsnpGraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed get connections without keys from graph: {:?}",
			error
		)))
	})
}

// Get one sided private friendship connections
#[no_mangle]
pub unsafe extern "C" fn graph_get_one_sided_private_friendship_connections(
	graph_state: *mut GraphState,
	user_id: *const DsnpUserId,
) -> DsnpGraphFFIResult<GraphConnections, DsnpGraphError> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return DsnpGraphFFIResult::new_error(DsnpGraphError::FFIError(
				"Graph state is null".to_string(),
			))
		}
		let graph_state = &mut *graph_state;
		let user_id = &*user_id;
		match graph_state.get_one_sided_private_friendship_connections(user_id) {
			Ok(connections) => {
				let connections_len = connections.len();
				let connections_ptr = ManuallyDrop::new(connections).as_mut_ptr();
				let graph_connections =
					GraphConnections { connections: connections_ptr, connections_len };
				DsnpGraphFFIResult::new(graph_connections)
			},
			Err(error) => DsnpGraphFFIResult::new_error(error),
		}
	});
	result.unwrap_or_else(|error| {
		DsnpGraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed get one sided private friendship connections from graph: {:?}",
			error
		)))
	})
}

// Get a list published and imported public keys associated with a user
#[no_mangle]
pub unsafe extern "C" fn graph_get_public_keys(
	graph_state: *mut GraphState,
	user_id: *const DsnpUserId,
) -> DsnpGraphFFIResult<DsnpPublicKeys, DsnpGraphError> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return DsnpGraphFFIResult::new_error(DsnpGraphError::FFIError(
				"Graph state is null".to_string(),
			))
		}
		let graph_state = &mut *graph_state;
		let user_id = &*user_id;

		match graph_state.get_public_keys(user_id) {
			Ok(keys) => {
				let ffi_keys = dsnp_public_keys_to_ffi(keys);
				let keys_len = ffi_keys.len();
				let keys_ptr = ManuallyDrop::new(ffi_keys).as_mut_ptr();
				let public_keys = DsnpPublicKeys { keys: keys_ptr, keys_len };
				DsnpGraphFFIResult::new(public_keys)
			},
			Err(error) => DsnpGraphFFIResult::new_error(error),
		}
	});
	result.unwrap_or_else(|error| {
		DsnpGraphFFIResult::new_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed get public keys from graph: {:?}",
			error
		)))
	})
}

// free graph state
#[no_mangle]
pub unsafe extern "C" fn free_graph_state(graph_state: *mut GraphState) {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return
		}
		let mut graph_states = GRAPH_STATES.lock().unwrap();
		graph_states.retain(|x| !std::ptr::eq(x.as_ref(), unsafe { &*graph_state }));
	});
	result.unwrap_or(());
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
