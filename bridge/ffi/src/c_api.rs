use crate::{bindings::*, utils::*, FFIResult, GraphError};
use dsnp_graph_config::{errors::DsnpGraphError, SchemaId};
use dsnp_graph_core::{
	api::api::{GraphAPI, GraphState},
	dsnp::dsnp_types::DsnpUserId,
};
use std::{
	ffi::{c_char, CString},
	mem::ManuallyDrop,
	panic,
	sync::Mutex,
};

#[no_mangle]
pub extern "C" fn print_hello_graph() {
	println!("Hello, Graph!");
}

// Collection of GraphStates
#[allow(clippy::vec_box)]
static GRAPH_STATES: Mutex<Vec<Box<GraphState>>> = Mutex::new(Vec::new());

/// Get the graph config for the given environment
/// # Safety
/// This function is unsafe because it dereferences a raw pointer
/// # Arguments
/// * `environment` - a pointer to an environment
/// # Returns
/// * `Config` - the graph config
/// # Errors
/// * `GraphError` - if the graph config cannot be retrieved
#[no_mangle]
pub unsafe extern "C" fn get_graph_config(
	environment: *const Environment,
) -> FFIResult<Config, GraphError> {
	let result = panic::catch_unwind(|| {
		let env = &*environment;
		let config_for_ffi = get_config_for_ffi(env);
		FFIResult::new(config_for_ffi)
	});

	result.unwrap_or_else(|error| {
		FFIResult::new_mut_error(GraphError::from_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to get graph config: {:?}",
			error
		))))
	})
}

/// Initialize a graph state with the given environment
/// # Safety
/// This function is unsafe because it dereferences a raw pointer
/// # Arguments
/// * `environment` - a pointer to an environment
/// # Returns
/// * `GraphState` - the pointer to the graph state
/// # Errors
/// * `GraphError` - if the graph state cannot be initialized
#[no_mangle]
pub unsafe extern "C" fn initialize_graph_state(
	environment: *const Environment,
) -> FFIResult<GraphState, GraphError> {
	let result = panic::catch_unwind(|| {
		let environment = &*environment;
		let rust_environment = environment_from_ffi(environment);
		let graph_state = Box::new(GraphState::new(rust_environment));
		let graph_state_ptr = Box::into_raw(graph_state);
		let mut graph_states = GRAPH_STATES.lock().unwrap();
		graph_states.push(Box::from_raw(graph_state_ptr));
		FFIResult::new_mut(graph_state_ptr)
	});
	result.unwrap_or_else(|error| {
		FFIResult::new_mut_error(GraphError::from_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to initialize graph state: {:?}",
			error
		))))
	})
}

/// Intialize GraphState with capacity and environment
/// # Safety
/// This function is unsafe because it dereferences a raw pointer
/// # Arguments
/// * `environment` - a pointer to an environment
/// * `capacity` - the capacity of the graph state
/// # Returns
/// * `GraphState` - the pointer to the graph state
/// # Errors
/// * `GraphError` - if the graph state cannot be initialized
#[no_mangle]
pub unsafe extern "C" fn initialize_graph_state_with_capacity(
	environment: *const Environment,
	capacity: usize,
) -> FFIResult<GraphState, GraphError> {
	let result = panic::catch_unwind(|| {
		let environment = &*environment;
		let rust_environment = environment_from_ffi(environment);
		let graph_state = Box::new(GraphState::with_capacity(rust_environment, capacity));
		let graph_state_ptr = Box::into_raw(graph_state);
		let mut graph_states = GRAPH_STATES.lock().unwrap();
		graph_states.push(Box::from_raw(graph_state_ptr));
		FFIResult::new_mut(graph_state_ptr)
	});
	result.unwrap_or_else(|error| {
		FFIResult::new_mut_error(GraphError::from_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to initialize graph state with capacity: {:?}",
			error
		))))
	})
}

/// Get capacity of graph state
/// # Safety
/// This function is unsafe because it dereferences a raw pointer
/// # Arguments
/// * `graph_state` - a pointer to a graph state
/// # Returns
/// * `usize` - the capacity of the graph state
/// # Errors
/// * `GraphError` - if the graph state fails to get capacity
#[no_mangle]
pub unsafe extern "C" fn get_graph_capacity(
	graph_state: *mut GraphState,
) -> FFIResult<usize, GraphError> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return FFIResult::new_mut_error(GraphError::from_error(DsnpGraphError::FFIError(
				"Graph state is null".to_string(),
			)))
		}
		let graph_state = &mut *graph_state;
		FFIResult::new(graph_state.capacity())
	});
	result.unwrap_or_else(|error| {
		FFIResult::new_mut_error(GraphError::from_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to get graph capacity: {:?}",
			error
		))))
	})
}

/// Get total graph states in GRAPH_STATES collection
/// # Returns
/// * `usize` - the total graph states in GRAPH_STATES collection
/// # Errors
/// * `GraphError` - if the count of graph states cannot be retrieved
#[no_mangle]
pub unsafe extern "C" fn get_graph_states_count() -> FFIResult<usize, GraphError> {
	let result = panic::catch_unwind(|| {
		let graph_states = GRAPH_STATES.lock().unwrap();
		FFIResult::new(graph_states.len())
	});
	result.unwrap_or_else(|error| {
		FFIResult::new_mut_error(GraphError::from_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to get graph states count: {:?}",
			error
		))))
	})
}

/// Check if a given state contains user graph
/// # Safety
/// This function is unsafe because it dereferences a raw pointer
/// # Arguments
/// * `graph_state` - a pointer to a graph state
/// * `user_id` - a pointer to a user id
/// # Returns
/// * `bool` - true if the user graph exists, false otherwise
/// # Errors
/// * `GraphError` - if state fails to check if user graph exists
#[no_mangle]
pub unsafe extern "C" fn graph_contains_user(
	graph_state: *mut GraphState,
	user_id: *const DsnpUserId,
) -> FFIResult<bool, GraphError> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return FFIResult::new_mut_error(GraphError::from_error(DsnpGraphError::FFIError(
				"Graph state is null".to_string(),
			)))
		}
		let graph_state = &mut *graph_state;
		let user_id = &*user_id;
		FFIResult::new(graph_state.contains_user_graph(user_id))
	});
	result.unwrap_or_else(|error| {
		FFIResult::new_mut_error(GraphError::from_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to check graph state for user: {:?}",
			error
		))))
	})
}

/// Count of users in current graph state
/// # Safety
/// This function is unsafe because it dereferences a raw pointer
/// # Arguments
/// * `graph_state` - a pointer to a graph state
/// # Returns
/// * `usize` - the count of users in the graph state
/// # Errors
/// * `GraphError` - if state fails to get users count
#[no_mangle]
pub unsafe extern "C" fn graph_users_count(
	graph_state: *mut GraphState,
) -> FFIResult<usize, GraphError> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return FFIResult::new_mut_error(GraphError::from_error(DsnpGraphError::FFIError(
				"Graph state is null".to_string(),
			)))
		}
		let graph_state = &mut *graph_state;
		FFIResult::new(graph_state.len())
	});
	result.unwrap_or_else(|error| {
		FFIResult::new_mut_error(GraphError::from_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to get users count from graph: {:?}",
			error
		))))
	})
}

/// Remove user from graph state
/// # Safety
/// This function is unsafe because it dereferences a raw pointer
/// # Arguments
/// * `graph_state` - a pointer to a graph state
/// * `user_id` - a pointer to a user id
/// # Returns
/// * `bool` - true if the user was removed, false otherwise
/// # Errors
/// * `GraphError` - if the graph state fails to remove user
#[no_mangle]
pub unsafe extern "C" fn graph_remove_user(
	graph_state: *mut GraphState,
	user_id: *const DsnpUserId,
) -> FFIResult<bool, GraphError> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return FFIResult::new_mut_error(GraphError::from_error(DsnpGraphError::FFIError(
				"Graph state is null".to_string(),
			)))
		}
		let graph_state = &mut *graph_state;
		let user_id = &*user_id;
		graph_state.remove_user_graph(user_id);
		FFIResult::new(true)
	});
	result.unwrap_or_else(|error| {
		FFIResult::new_mut_error(GraphError::from_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to remove user from graph: {:?}",
			error
		))))
	})
}

/// Import users data to graph state
/// # Safety
/// This function is unsafe because it dereferences a raw pointer
/// # Arguments
/// * `graph_state` - a pointer to a graph state
/// * `payloads` - a pointer to an array of payloads
/// * `payloads_len` - the length of the payloads array
/// # Returns
/// * `bool` - true if the users data was imported, false otherwise
/// # Errors
/// * `GraphError` - if the graph state fails to import users data
#[no_mangle]
pub unsafe extern "C" fn graph_import_users_data(
	graph_state: *mut GraphState,
	payloads: *const ImportBundle,
	payloads_len: usize,
) -> FFIResult<bool, GraphError> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return FFIResult::new_mut_error(GraphError::from_error(DsnpGraphError::FFIError(
				"Graph state is null".to_string(),
			)))
		}
		let graph_state = &mut *graph_state;
		let payloads = std::slice::from_raw_parts(payloads, payloads_len);
		let payloads = payloads_from_ffi(&payloads);
		let imported = graph_state.import_users_data(&payloads);
		match imported {
			Ok(_) => FFIResult::new(true),
			Err(error) => FFIResult::new_mut_error(GraphError::from_error(error)),
		}
	});
	result.unwrap_or_else(|error| {
		FFIResult::new_mut_error(GraphError::from_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to import users data to graph: {:?}",
			error
		))))
	})
}

/// Export updates from graph state
/// # Safety
/// This function is unsafe because it dereferences a raw pointer
/// # Arguments
/// * `graph_state` - a pointer to a graph state
/// # Returns
/// * `GraphUpdates` - the pointer to the graph updates
/// # Errors
/// * `GraphError` - if the graph updates cannot be retrieved
#[no_mangle]
pub unsafe extern "C" fn graph_export_updates(
	graph_state: *mut GraphState,
) -> FFIResult<GraphUpdates, GraphError> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return FFIResult::new_mut_error(GraphError::from_error(DsnpGraphError::FFIError(
				"Graph state is null".to_string(),
			)))
		}
		let graph_state = &mut *graph_state;
		match graph_state.export_updates() {
			Ok(updates) => {
				let ffi_updates = updates_to_ffi(updates);
				let updates_len = ffi_updates.len();
				let updates_ptr = ManuallyDrop::new(ffi_updates).as_mut_ptr();
				let graph_updates = GraphUpdates { updates: updates_ptr, updates_len };
				FFIResult::new(graph_updates)
			},
			Err(error) => FFIResult::new_mut_error(GraphError::from_error(error)),
		}
	});
	result.unwrap_or_else(|error| {
		FFIResult::new_mut_error(GraphError::from_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to export updates from graph: {:?}",
			error
		))))
	})
}

/// Apply actions to graph state
/// # Safety
/// This function is unsafe because it dereferences a raw pointer
/// # Arguments
/// * `graph_state` - a pointer to a graph state
/// * `actions` - a pointer to an array of actions
/// * `actions_len` - the length of the actions array
/// # Returns
/// * `bool` - true if the actions were applied, false otherwise
/// # Errors
/// * `GraphError` - if the actions cannot be applied to the graph state
#[no_mangle]
pub unsafe extern "C" fn graph_apply_actions(
	graph_state: *mut GraphState,
	actions: *const Action,
	actions_len: usize,
) -> FFIResult<bool, GraphError> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return FFIResult::new_mut_error(GraphError::from_error(DsnpGraphError::FFIError(
				"Graph state is null".to_string(),
			)))
		}
		let graph_state = &mut *graph_state;
		let actions = std::slice::from_raw_parts(actions, actions_len);
		let actions = actions_from_ffi(&actions);
		match graph_state.apply_actions(&actions) {
			Ok(_) => FFIResult::new(true),
			Err(error) => FFIResult::new_mut_error(GraphError::from_error(error)),
		}
	});
	result.unwrap_or_else(|error| {
		FFIResult::new_mut_error(GraphError::from_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to apply actions to graph: {:?}",
			error
		))))
	})
}

/// Get connections for user from graph state
/// # Safety
/// This function is unsafe because it dereferences a raw pointer
/// # Arguments
/// * `graph_state` - a pointer to a graph state
/// * `user_id` - a pointer to a user id
/// * `schema_id` - a pointer to a schema id
/// * `include_pending` - a boolean to include pending connections
/// # Returns
/// * `GraphConnections` - the pointer to the graph connections
/// # Errors
/// * `GraphError` - if the connections cannot be retrieved
#[no_mangle]
pub unsafe extern "C" fn graph_get_connections_for_user(
	graph_state: *mut GraphState,
	user_id: *const DsnpUserId,
	schema_id: *const SchemaId,
	include_pending: bool,
) -> FFIResult<GraphConnections, GraphError> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return FFIResult::new_mut_error(GraphError::from_error(DsnpGraphError::FFIError(
				"Graph state is null".to_string(),
			)))
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
				FFIResult::new(graph_connections)
			},
			Err(error) => FFIResult::new_mut_error(GraphError::from_error(error)),
		}
	});
	result.unwrap_or_else(|error| {
		FFIResult::new_mut_error(GraphError::from_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed get connections for user from graph: {:?}",
			error
		))))
	})
}

/// Get user connections without keys from graph state
/// # Safety
/// This function is unsafe because it dereferences a raw pointer
/// # Arguments
/// * `graph_state` - a pointer to a graph state
/// # Returns
/// * `GraphConnectionsWithoutKeys` - the pointer to the graph connections without keys
/// # Errors
/// * `GraphError` - if the connections cannot be retrieved
#[no_mangle]
pub unsafe extern "C" fn graph_get_connections_without_keys(
	graph_state: *mut GraphState,
) -> FFIResult<GraphConnectionsWithoutKeys, GraphError> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return FFIResult::new_mut_error(GraphError::from_error(DsnpGraphError::FFIError(
				"Graph state is null".to_string(),
			)))
		}
		let graph_state = &mut *graph_state;
		match graph_state.get_connections_without_keys() {
			Ok(connections) => {
				let connections_len = connections.len();
				let connections_ptr = ManuallyDrop::new(connections).as_mut_ptr();
				let graph_connections =
					GraphConnectionsWithoutKeys { connections: connections_ptr, connections_len };
				FFIResult::new(graph_connections)
			},
			Err(error) => FFIResult::new_mut_error(GraphError::from_error(error)),
		}
	});
	result.unwrap_or_else(|error| {
		FFIResult::new_mut_error(GraphError::from_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed get connections without keys from graph: {:?}",
			error
		))))
	})
}

/// Get one sided private friendship connections for a user from graph state
/// # Safety
/// This function is unsafe because it dereferences a raw pointer
/// # Arguments
/// * `graph_state` - a pointer to a graph state
/// * `user_id` - a pointer to a user id
/// # Returns
/// * `GraphConnections` - the pointer to the graph connections
/// # Errors
/// * `GraphError` - if the connections cannot be retrieved
#[no_mangle]
pub unsafe extern "C" fn graph_get_one_sided_private_friendship_connections(
	graph_state: *mut GraphState,
	user_id: *const DsnpUserId,
) -> FFIResult<GraphConnections, GraphError> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return FFIResult::new_mut_error(GraphError::from_error(DsnpGraphError::FFIError(
				"Graph state is null".to_string(),
			)))
		}
		let graph_state = &mut *graph_state;
		let user_id = &*user_id;
		match graph_state.get_one_sided_private_friendship_connections(user_id) {
			Ok(connections) => {
				let connections_len = connections.len();
				let connections_ptr = ManuallyDrop::new(connections).as_mut_ptr();
				let graph_connections =
					GraphConnections { connections: connections_ptr, connections_len };
				FFIResult::new(graph_connections)
			},
			Err(error) => FFIResult::new_mut_error(GraphError::from_error(error)),
		}
	});
	result.unwrap_or_else(|error| {
		FFIResult::new_mut_error(GraphError::from_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed get one sided private friendship connections from graph: {:?}",
			error
		))))
	})
}

/// Get a list of published and imported public keys associated with a user
/// # Safety
/// This function is unsafe because it dereferences a raw pointer
/// # Arguments
/// * `graph_state` - a pointer to the graph state
/// * `user_id` - a pointer to a user id
/// # Returns
/// * `DsnpPublicKeys` - the pointer to the public keys
/// # Errors
/// * `GraphError` - if the public keys cannot be retrieved
#[no_mangle]
pub unsafe extern "C" fn graph_get_public_keys(
	graph_state: *mut GraphState,
	user_id: *const DsnpUserId,
) -> FFIResult<DsnpPublicKeys, GraphError> {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return FFIResult::new_mut_error(GraphError::from_error(DsnpGraphError::FFIError(
				"Graph state is null".to_string(),
			)))
		}
		let graph_state = &mut *graph_state;
		let user_id = &*user_id;

		match graph_state.get_public_keys(user_id) {
			Ok(keys) => {
				let ffi_keys = dsnp_public_keys_to_ffi(keys);
				let keys_len = ffi_keys.len();
				let keys_ptr = ManuallyDrop::new(ffi_keys).as_mut_ptr();
				let public_keys = DsnpPublicKeys { keys: keys_ptr, keys_len };
				FFIResult::new(public_keys)
			},
			Err(error) => FFIResult::new_mut_error(GraphError::from_error(error)),
		}
	});
	result.unwrap_or_else(|error| {
		FFIResult::new_mut_error(GraphError::from_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed get public keys from graph: {:?}",
			error
		))))
	})
}

/// Returns the deserialized dsnp keys
/// # Safety
/// This function is unsafe because it dereferences a raw pointer
/// # Arguments
/// * `dsnp_keys` - a pointer to the dsnp keys
/// # Returns
/// * `DsnpPublicKeys` - the pointer to the public keys
/// # Errors
/// * `GraphError` - if the public keys cannot be retrieved
#[no_mangle]
pub unsafe extern "C" fn graph_deserialize_dsnp_keys(
	dsnp_keys: *const DsnpKeys,
) -> FFIResult<DsnpPublicKeys, GraphError> {
	let result = panic::catch_unwind(|| {
		let dsnp_keys = &*dsnp_keys;
		let rust_dsnp_keys = dsnp_keys_from_ffi(dsnp_keys);
		match GraphState::deserialize_dsnp_keys(&rust_dsnp_keys) {
			Ok(keys) => {
				let ffi_keys = dsnp_public_keys_to_ffi(keys);
				let keys_len = ffi_keys.len();
				let keys_ptr = ManuallyDrop::new(ffi_keys).as_mut_ptr();
				let public_keys = DsnpPublicKeys { keys: keys_ptr, keys_len };
				FFIResult::new(public_keys)
			},
			Err(error) => FFIResult::new_mut_error(GraphError::from_error(error)),
		}
	});
	result.unwrap_or_else(|error| {
		FFIResult::new_mut_error(GraphError::from_error(DsnpGraphError::Unknown(anyhow::anyhow!(
			"Failed to deserialized dsnp keys: {:?}",
			error
		))))
	})
}

/// Free graph state
/// # Arguments
/// * `graph_state` - a pointer to the graph state
#[no_mangle]
pub unsafe extern "C" fn free_graph_state(graph_state: *mut GraphState) {
	let result = panic::catch_unwind(|| {
		if graph_state.is_null() {
			return
		}
		let mut graph_states = GRAPH_STATES.lock().unwrap();
		let index =
			graph_states.iter().position(|x| x.as_ref() as *const _ == graph_state).unwrap();
		graph_states.remove(index);
	});
	result.unwrap_or(())
}

/// Free GraphStates
/// # Errors
/// * `GraphError` - if the graph states cannot be freed
#[no_mangle]
pub extern "C" fn free_graph_states() {
	let result = panic::catch_unwind(|| {
		let mut graph_states = GRAPH_STATES.lock().unwrap();
		graph_states.clear();
	});
	result.unwrap_or(())
}

/// Free GraphUpdates
/// # Arguments
/// * `graph_updates` - a pointer to the graph updates
#[no_mangle]
pub unsafe extern "C" fn free_graph_updates(graph_updates: *mut GraphUpdates) {
	let result = panic::catch_unwind(|| {
		let _ = Box::from_raw(graph_updates);
	});
	result.unwrap_or(())
}

/// Free GraphConnections
/// # Arguments
/// * `graph_connections` - a pointer to the graph connections
#[no_mangle]
pub unsafe extern "C" fn free_graph_connections(graph_connections: *mut GraphConnections) {
	let result = panic::catch_unwind(|| {
		let _ = Box::from_raw(graph_connections);
	});
	result.unwrap_or(())
}

/// Free GraphConnectionsWithoutKeys
/// # Arguments
/// * `graph_connections` - a pointer to the graph connections
#[no_mangle]
pub unsafe extern "C" fn free_graph_connections_without_keys(
	graph_connections: *mut GraphConnectionsWithoutKeys,
) {
	let result = panic::catch_unwind(|| {
		let _ = Box::from_raw(graph_connections);
	});
	result.unwrap_or(())
}

/// Free DsnpPublicKeys
/// # Arguments
/// * `public_keys` - a pointer to the public keys
#[no_mangle]
pub unsafe extern "C" fn free_graph_dsnp_public_keys(public_keys: *mut DsnpPublicKeys) {
	let result = panic::catch_unwind(|| {
		let _ = Box::from_raw(public_keys);
	});
	result.unwrap_or(())
}

/// Free GraphError
/// # Arguments
/// * `error` - a pointer to the graph error
#[no_mangle]
pub unsafe extern "C" fn free_dsnp_graph_error(error: *mut GraphError) {
	if !error.is_null() {
		unsafe {
			let _ = Box::from_raw(error);
		}
	}
}

/// Get error code from graph error
/// # Arguments
/// * `error` - a pointer to the graph error
/// # Returns
/// * `i32` - the error code or std::i32::MAX
#[no_mangle]
pub unsafe extern "C" fn dsnp_graph_error_code(error: *const GraphError) -> i32 {
	if let Some(error) = unsafe { error.as_ref() } {
		error.error_code()
	} else {
		std::i32::MAX
	}
}

/// Get error message from graph error
/// # Arguments
/// * `error` - a pointer to the graph error
/// # Returns
/// * `*const c_char` - the error message or null
#[no_mangle]
pub unsafe extern "C" fn dsnp_graph_error_message(error: *const GraphError) -> *const c_char {
	if let Some(error) = unsafe { error.as_ref() } {
		let error_msg = CString::new(error.error_message()).unwrap_or_default();
		error_msg.into_raw()
	} else {
		std::ptr::null()
	}
}

/// Free error message
/// # Arguments
/// * `error_message` - a pointer to the error message
#[no_mangle]
pub unsafe extern "C" fn free_dsnp_graph_error_message(error_message: *const c_char) {
	if !error_message.is_null() {
		unsafe {
			let _ = CString::from_raw(error_message as *mut c_char);
		}
	}
}

/// Free graph config
/// # Arguments
/// * `config` - a pointer to the graph config
#[no_mangle]
pub unsafe extern "C" fn free_graph_config(config: *mut Config) {
	let result = panic::catch_unwind(|| {
		let _ = Box::from_raw(config);
	});
	result.unwrap_or(())
}
