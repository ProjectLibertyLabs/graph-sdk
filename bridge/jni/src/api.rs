use crate::{
	errors::SdkJniError,
	helper::{handle_result, validate_handle},
	mappings::{
		convert_jboolean, map_to_actions, map_to_dsnp_keys, map_to_environment, map_to_imports,
		serialize_config, serialize_dsnp_users, serialize_graph_edges, serialize_graph_keypair,
		serialize_graph_updates, serialize_public_keys,
	},
};
use dsnp_graph_config::{DsnpUserId, GraphKeyType, SchemaId};
use dsnp_graph_core::{
	api::api::{GraphAPI, GraphState},
	util::transactional_hashmap::Transactional,
};
use jni::{
	objects::{JByteArray, JClass, JObject, JString},
	sys::{jboolean, jint, jlong},
	JNIEnv,
};
use std::{
	ops::{Deref, DerefMut},
	panic,
	sync::RwLock,
};

pub type SdkJniResult<V> = Result<V, SdkJniError>;

// Collection of GraphStates memory locations
static GRAPH_STATES_MEMORY_LOCATIONS: RwLock<Vec<jlong>> = RwLock::new(Vec::new());

#[no_mangle]
pub extern "C" fn Java_io_amplica_graphsdk_Native_hello<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	input: JString<'local>,
) -> JString<'local> {
	let input: String = env.get_string(&input).expect("Couldn't get java string!").into();

	let output = env
		.new_string(format!("Hello, {}!", input))
		.expect("Couldn't create java string!");
	output
}

/// An optimization barrier / guard against garbage collection.
///
/// cbindgen:ignore
#[no_mangle]
pub extern "C" fn Java_io_amplica_graphsdk_Native_keepAlive<'local>(
	mut _env: JNIEnv<'local>,
	_class: JClass<'local>,
	_input: JObject<'local>,
) {
}

/// Initializes the graph state and returns a handle to it.
/// The handle is a pointer to the memory location of the state.
/// The memory will be freed when `freeGraphState` is called.
/// # Arguments
/// * `environment` - the environment to initialize the graph state with
/// # Returns
/// * `jlong` - the handle to the graph state
/// # Errors
/// * `SdkJniError` - if initializing graph state fails
#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_initializeGraphState<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	environment: JByteArray,
) -> jlong {
	let result = panic::catch_unwind(|| {
		let rust_environment = map_to_environment(&env, &environment)?;
		let graph_state = GraphState::new(rust_environment);
		let boxed = Box::new(graph_state);
		let mut graph_states =
			GRAPH_STATES_MEMORY_LOCATIONS.write().map_err(|_| SdkJniError::LockError)?;

		// graph state memory will be handled manually after following line execution
		let handle = Box::into_raw(boxed) as jlong;
		graph_states.push(handle);
		Ok(handle)
	});
	handle_result(&mut env, result)
}

/// Frees the graph state memory.
/// # Arguments
/// * `handle` - the handle to the graph state
/// # Errors
/// * `SdkJniError` - if freeing graph state fails
#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_freeGraphState<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	handle: jlong,
) {
	let result = panic::catch_unwind(|| {
		if handle == 0 {
			return Err(SdkJniError::InvalidHandle("is null"));
		}
		let mut graph_states =
			GRAPH_STATES_MEMORY_LOCATIONS.write().map_err(|_| SdkJniError::LockError)?;
		let index = graph_states
			.iter()
			.position(|x| *x == handle)
			.ok_or(SdkJniError::InvalidHandle("does not exist"))?;
		graph_states.remove(index);

		// following line frees the allocated memory for state
		let _ = unsafe { Box::from_raw(handle as *mut GraphState) };
		Ok(())
	});
	handle_result(&mut env, result);
}

/// Get config for an environment.
/// # Arguments
/// * `environment` - the environment to get config for
/// # Returns
/// * `jbyteArray` - the serialized config
/// # Errors
/// * `SdkJniError` - if getting config fails
#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_getConfig<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	environment: JByteArray,
) -> JByteArray<'local> {
	let result = panic::catch_unwind(|| {
		let rust_environment = map_to_environment(&env, &environment)?;
		let config = rust_environment.get_config();
		let result = serialize_config(&env, &config)?;
		Ok(result)
	});
	handle_result(&mut env, result)
}

/// Check if user graph exists.
/// # Arguments
/// * `handle` - the handle to the graph state
/// * `dsnp_user_id` - the user id to check
/// # Returns
/// * `jboolean` - true if user graph exists
/// # Errors
/// * `SdkJniError` - if checking user graph fails
#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_containsUserGraph<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	handle: jlong,
	dsnp_user_id: jlong,
) -> jboolean {
	let result = panic::catch_unwind(|| {
		validate_handle(&GRAPH_STATES_MEMORY_LOCATIONS, handle)?;
		// TODO: test edge case that dsnp_user_id is bigger than i64
		let user_id = u64::try_from(dsnp_user_id)
			.map_err(|_| SdkJniError::BadJniParameter("invalid dsnp_user_id"))?;

		// locking to read from state
		let _lock = GRAPH_STATES_MEMORY_LOCATIONS.read().map_err(|_| SdkJniError::LockError)?;
		let graph = unsafe { Box::from_raw(handle as *mut GraphState) };
		let result = graph.deref().contains_user_graph(&user_id).into();

		// pulling out of the box as raw so that memory stays allocated
		let _ = Box::into_raw(graph) as jlong;
		Ok(result)
	});
	handle_result(&mut env, result)
}

/// Get user graph length.
/// # Arguments
/// * `handle` - the handle to the graph state
/// # Returns
/// * `jint` - the length of the user graph
/// # Errors
/// * `SdkJniError` - if getting user graph length fails
#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_getGraphUsersLength<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	handle: jlong,
) -> jint {
	let result = panic::catch_unwind(|| {
		validate_handle(&GRAPH_STATES_MEMORY_LOCATIONS, handle)?;

		// locking to read from state
		let _lock = GRAPH_STATES_MEMORY_LOCATIONS.read().map_err(|_| SdkJniError::LockError)?;
		let graph = unsafe { Box::from_raw(handle as *mut GraphState) };
		let result = graph.deref().len() as jint;

		// pulling out of the box as raw so that memory stays allocated
		let _ = Box::into_raw(graph) as jlong;
		Ok(result)
	});
	handle_result(&mut env, result)
}

/// Remove user from graph state.
/// # Arguments
/// * `handle` - the handle to the graph state
/// * `dsnp_user_id` - the user id to remove
/// # Errors
/// * `SdkJniError` - if removing user fails
#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_removeUserGraph<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	handle: jlong,
	dsnp_user_id: jlong,
) {
	let result = panic::catch_unwind(|| {
		validate_handle(&GRAPH_STATES_MEMORY_LOCATIONS, handle)?;
		let user_id = u64::try_from(dsnp_user_id)
			.map_err(|_| SdkJniError::BadJniParameter("invalid dsnp_user_id"))?;

		// locking to write in state
		let _lock = GRAPH_STATES_MEMORY_LOCATIONS.write().map_err(|_| SdkJniError::LockError)?;
		let mut graph = unsafe { Box::from_raw(handle as *mut GraphState) };
		graph.deref_mut().remove_user_graph(&user_id);

		// pulling out of the box as raw so that memory stays allocated
		let _ = Box::into_raw(graph) as jlong;
		Ok(())
	});
	handle_result(&mut env, result)
}

/// Import user data into graph state.
/// # Arguments
/// * `handle` - the handle to the graph state
/// * `imports` - the serialized imports
/// # Errors
/// * `SdkJniError` - if imports are invalid
#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_importUserData<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	handle: jlong,
	imports: JByteArray,
) {
	let result = panic::catch_unwind(|| {
		validate_handle(&GRAPH_STATES_MEMORY_LOCATIONS, handle)?;
		let rust_imports = map_to_imports(&env, &imports)?;

		// locking to write in state
		let _lock = GRAPH_STATES_MEMORY_LOCATIONS.write().map_err(|_| SdkJniError::LockError)?;
		let mut graph = unsafe { Box::from_raw(handle as *mut GraphState) };
		// do not use `?` here to handle the error since it would drop the memory
		let result = graph
			.deref_mut()
			.import_users_data(&rust_imports)
			.map_err(|e| SdkJniError::from(e));

		// pulling out of the box as raw so that memory stays allocated
		let _ = Box::into_raw(graph) as jlong;
		result
	});
	handle_result(&mut env, result)
}

/// Export updates to graph state.
/// # Arguments
/// * `handle` - the handle to the graph state
/// # Returns
/// * `jbyteArray` - the serialized updates
/// # Errors
/// * `SdkJniError` - if exporting updates fails
#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_exportUpdates<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	handle: jlong,
) -> JByteArray<'local> {
	let result = panic::catch_unwind(|| {
		validate_handle(&GRAPH_STATES_MEMORY_LOCATIONS, handle)?;

		// locking to read from state
		let _lock = GRAPH_STATES_MEMORY_LOCATIONS.read().map_err(|_| SdkJniError::LockError)?;
		let graph = unsafe { Box::from_raw(handle as *mut GraphState) };
		// do not use `?` here to handle the error since it would drop the memory
		let result = graph
			.deref()
			.export_updates()
			.map_err(|e| SdkJniError::from(e))
			.and_then(|updates| serialize_graph_updates(&env, &updates));

		// pulling out of the box as raw so that memory stays allocated
		let _ = Box::into_raw(graph) as jlong;
		result
	});
	handle_result(&mut env, result)
}

/// Export updates to graph state for a single user graph.
/// # Arguments
/// * `handle` - the handle to the graph state
/// * `dsnp_user_id` - the user id whose graph to export
/// # Returns
/// * `jbyteArray` - the serialized updates
/// # Errors
/// * `SdkJniError` - if exporting updates fails
#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_exportUserGraphUpdates<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	handle: jlong,
	dsnp_user_id: jlong,
) -> JByteArray<'local> {
	let result = panic::catch_unwind(|| {
		validate_handle(&GRAPH_STATES_MEMORY_LOCATIONS, handle)?;
		let dsnp_user_id = DsnpUserId::try_from(dsnp_user_id)
			.map_err(|_| SdkJniError::BadJniParameter("invalid dsnp_user_id"))?;

		// locking to read from state
		let _lock = GRAPH_STATES_MEMORY_LOCATIONS.read().map_err(|_| SdkJniError::LockError)?;
		let graph = unsafe { Box::from_raw(handle as *mut GraphState) };
		// do not use `?` here to handle the error since it would drop the memory
		let result = graph
			.deref()
			.export_user_graph_updates(&dsnp_user_id)
			.map_err(|e| SdkJniError::from(e))
			.and_then(|updates| serialize_graph_updates(&env, &updates));

		// pulling out of the box as raw so that memory stays allocated
		let _ = Box::into_raw(graph) as jlong;
		result
	});
	handle_result(&mut env, result)
}

/// Apply actions to graph state.
/// # Arguments
/// * `handle` - the handle to the graph state
/// * `actions` - the serialized actions
/// # Errors
/// * `SdkJniError` - if applying actions fails
#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_applyActions<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	handle: jlong,
	actions: JByteArray,
) {
	let result = panic::catch_unwind(|| {
		validate_handle(&GRAPH_STATES_MEMORY_LOCATIONS, handle)?;
		let (actions, options) = map_to_actions(&env, &actions)?;

		// locking to write in state
		let _lock = GRAPH_STATES_MEMORY_LOCATIONS.write().map_err(|_| SdkJniError::LockError)?;
		let mut graph = unsafe { Box::from_raw(handle as *mut GraphState) };
		// do not use `?` here to handle the error since it would drop the memory
		let result = graph
			.deref_mut()
			.apply_actions(&actions, &options)
			.map_err(|e| SdkJniError::from(e));

		// pulling out of the box as raw so that memory stays allocated
		let _ = Box::into_raw(graph) as jlong;
		result
	});
	handle_result(&mut env, result)
}

/// Commit pending actions to a graph state.
/// # Arguments
/// * `handle` - the handle to the graph state
/// * `actions` - the serialized actions
/// # Errors
/// * `SdkJniError` - if applying actions fails
#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_commit<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	handle: jlong,
) {
	let result = panic::catch_unwind(|| {
		validate_handle(&GRAPH_STATES_MEMORY_LOCATIONS, handle)?;

		// locking to write in state
		let _lock = GRAPH_STATES_MEMORY_LOCATIONS.write().map_err(|_| SdkJniError::LockError)?;
		let mut graph = unsafe { Box::from_raw(handle as *mut GraphState) };
		// do not use `?` here to handle the error since it would drop the memory
		graph.deref_mut().commit();

		// pulling out of the box as raw so that memory stays allocated
		let _ = Box::into_raw(graph) as jlong;
		Ok(())
	});
	handle_result(&mut env, result)
}

/// Rollback pending actions in a graph state.
/// # Arguments
/// * `handle` - the handle to the graph state
/// * `actions` - the serialized actions
/// # Errors
/// * `SdkJniError` - if applying actions fails
#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_rollback<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	handle: jlong,
) {
	let result = panic::catch_unwind(|| {
		validate_handle(&GRAPH_STATES_MEMORY_LOCATIONS, handle)?;

		// locking to write in state
		let _lock = GRAPH_STATES_MEMORY_LOCATIONS.write().map_err(|_| SdkJniError::LockError)?;
		let mut graph = unsafe { Box::from_raw(handle as *mut GraphState) };
		// do not use `?` here to handle the error since it would drop the memory
		graph.deref_mut().rollback();

		// pulling out of the box as raw so that memory stays allocated
		let _ = Box::into_raw(graph) as jlong;
		Ok(())
	});
	handle_result(&mut env, result)
}

/// Force calculate graph updates.
/// # Arguments
/// * `handle` - the handle to the graph state
/// * `dsnp_user_id` - the user id to calculate updates for
/// # Returns
/// * `jbyteArray` - the serialized updates
/// # Errors
/// * `SdkJniError` - if calculating updates fails
#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_forceCalculateGraphs<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	handle: jlong,
	dsnp_user_id: jlong,
) -> JByteArray<'local> {
	let result = panic::catch_unwind(|| {
		validate_handle(&GRAPH_STATES_MEMORY_LOCATIONS, handle)?;
		let dsnp_user_id = DsnpUserId::try_from(dsnp_user_id)
			.map_err(|_| SdkJniError::BadJniParameter("invalid dsnp_user_id"))?;

		// locking to read from state
		let _lock = GRAPH_STATES_MEMORY_LOCATIONS.read().map_err(|_| SdkJniError::LockError)?;
		let graph = unsafe { Box::from_raw(handle as *mut GraphState) };
		// do not use `?` here to handle the error since it would drop the memory
		let result = graph
			.deref()
			.force_recalculate_graphs(&dsnp_user_id)
			.map_err(|e| SdkJniError::from(e))
			.and_then(|updates| serialize_graph_updates(&env, &updates));

		// pulling out of the box as raw so that memory stays allocated
		let _ = Box::into_raw(graph) as jlong;
		result
	});
	handle_result(&mut env, result)
}

/// Get connections for a user graph.
/// # Arguments
/// * `handle` - the handle to the graph state
/// * `dsnp_user_id` - the user id to get connections for
/// * `schema_id` - the schema id to get connections for
/// * `include_pending` - whether to include pending connections
/// # Returns
/// * `jbyteArray` - the serialized connections
/// # Errors
/// * `SdkJniError` - if getting connections fails
#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_getConnectionsForUserGraph<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	handle: jlong,
	dsnp_user_id: jlong,
	schema_id: jint,
	include_pending: jboolean,
) -> JByteArray<'local> {
	let result = panic::catch_unwind(|| {
		validate_handle(&GRAPH_STATES_MEMORY_LOCATIONS, handle)?;
		let dsnp_user_id = DsnpUserId::try_from(dsnp_user_id)
			.map_err(|_| SdkJniError::BadJniParameter("invalid dsnp_user_id"))?;
		let schema_id = SchemaId::try_from(schema_id)
			.map_err(|_| SdkJniError::BadJniParameter("invalid schema_id"))?;
		let include_pending = convert_jboolean(include_pending)
			.map_err(|_| SdkJniError::BadJniParameter("invalid include_pending"))?;

		// locking to read from state
		let _lock = GRAPH_STATES_MEMORY_LOCATIONS.read().map_err(|_| SdkJniError::LockError)?;
		let graph = unsafe { Box::from_raw(handle as *mut GraphState) };
		// do not use `?` here to handle the error since it would drop the memory
		let result = graph
			.deref()
			.get_connections_for_user_graph(&dsnp_user_id, &schema_id, include_pending)
			.map_err(|e| SdkJniError::from(e))
			.and_then(|graph_edges| serialize_graph_edges(&env, &graph_edges));

		// pulling out of the box as raw so that memory stays allocated
		let _ = Box::into_raw(graph) as jlong;
		result
	});
	handle_result(&mut env, result)
}

/// Get users connections without keys.
/// # Arguments
/// * `handle` - the handle to the graph state
/// # Returns
/// * `jbyteArray` - the serialized users
/// # Errors
/// * `SdkJniError` - if getting users fails
#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_getUsersWithoutKeys<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	handle: jlong,
) -> JByteArray<'local> {
	let result = panic::catch_unwind(|| {
		validate_handle(&GRAPH_STATES_MEMORY_LOCATIONS, handle)?;

		// locking to read from state
		let _lock = GRAPH_STATES_MEMORY_LOCATIONS.read().map_err(|_| SdkJniError::LockError)?;
		let graph = unsafe { Box::from_raw(handle as *mut GraphState) };
		// do not use `?` here to handle the error since it would drop the memory
		let result = graph
			.deref()
			.get_connections_without_keys()
			.map_err(|e| SdkJniError::from(e))
			.and_then(|dsnp_users| serialize_dsnp_users(&env, &dsnp_users));

		// pulling out of the box as raw so that memory stays allocated
		let _ = Box::into_raw(graph) as jlong;
		result
	});
	handle_result(&mut env, result)
}

/// Get one sided private friendship connections.
/// # Arguments
/// * `handle` - the handle to the graph state
/// * `dsnp_user_id` - the user id to get connections for
/// # Returns
/// * `jbyteArray` - the serialized connections
/// # Errors
/// * `SdkJniError` - if getting connections fails
#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_getOneSidedPrivateFriendshipConnections<
	'local,
>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	handle: jlong,
	dsnp_user_id: jlong,
) -> JByteArray<'local> {
	let result = panic::catch_unwind(|| {
		validate_handle(&GRAPH_STATES_MEMORY_LOCATIONS, handle)?;
		let user_id = u64::try_from(dsnp_user_id)
			.map_err(|_| SdkJniError::BadJniParameter("invalid dsnp_user_id"))?;

		// locking to read from state
		let _lock = GRAPH_STATES_MEMORY_LOCATIONS.read().map_err(|_| SdkJniError::LockError)?;
		let graph = unsafe { Box::from_raw(handle as *mut GraphState) };
		// do not use `?` here to handle the error since it would drop the memory
		let result = graph
			.deref()
			.get_one_sided_private_friendship_connections(&user_id)
			.map_err(|e| SdkJniError::from(e))
			.and_then(|graph_edges| serialize_graph_edges(&env, &graph_edges));

		// pulling out of the box as raw so that memory stays allocated
		let _ = Box::into_raw(graph) as jlong;
		result
	});
	handle_result(&mut env, result)
}

/// Get public keys for a user.
/// # Arguments
/// * `handle` - the handle to the graph state
/// * `dsnp_user_id` - the user id to get public keys for
/// # Returns
/// * `jbyteArray` - the serialized public keys
/// # Errors
/// * `SdkJniError` - if getting public keys fails
#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_getPublicKeys<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	handle: jlong,
	dsnp_user_id: jlong,
) -> JByteArray<'local> {
	let result = panic::catch_unwind(|| {
		validate_handle(&GRAPH_STATES_MEMORY_LOCATIONS, handle)?;
		let user_id = u64::try_from(dsnp_user_id)
			.map_err(|_| SdkJniError::BadJniParameter("invalid dsnp_user_id"))?;

		// locking to read from state
		let _lock = GRAPH_STATES_MEMORY_LOCATIONS.read().map_err(|_| SdkJniError::LockError)?;
		let graph = unsafe { Box::from_raw(handle as *mut GraphState) };
		// do not use `?` here to handle the error since it would drop the memory
		let result = graph
			.deref()
			.get_public_keys(&user_id)
			.map_err(|e| SdkJniError::from(e))
			.and_then(|public_keys| serialize_public_keys(&env, &public_keys));

		// pulling out of the box as raw so that memory stays allocated
		let _ = Box::into_raw(graph) as jlong;
		result
	});
	handle_result(&mut env, result)
}

/// Deserialize DSNP keys.
/// # Arguments
/// * `dsnp_keys` - the serialized DSNP keys
/// # Returns
/// * `jbyteArray` - the deserialized public keys
/// # Errors
/// * `SdkJniError` - if deserializing DSNP keys fails
#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_deserializeDsnpKeys<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	dsnp_keys: JByteArray,
) -> JByteArray<'local> {
	let result = panic::catch_unwind(|| {
		let rust_dsnp_keys = map_to_dsnp_keys(&env, &dsnp_keys)?;
		GraphState::deserialize_dsnp_keys(&rust_dsnp_keys)
			.map_err(|e| SdkJniError::from(e))
			.and_then(|public_keys| serialize_public_keys(&env, &public_keys))
	});
	handle_result(&mut env, result)
}

/// Generate GraphKeyPair for a given GraphKeyType.
/// # Arguments
/// * `graph_key_type` - the type of the key to generate
/// # Returns
/// * `jbyteArray` - the serialized GraphKeyPair
/// # Errors
/// * `SdkJniError` - if generating GraphKeyPair fails
/// * `SdkJniError` - if GraphKeyType is InvalidHandle
#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_generateKeyPair<'local>(
	mut env: JNIEnv<'local>,
	_class: JClass<'local>,
	graph_key_type: jint,
) -> JByteArray<'local> {
	let result = panic::catch_unwind(|| {
		let key_type = u8::try_from(graph_key_type)
			.map_err(|_| SdkJniError::BadJniParameter("invalid graph_key_type"))?;

		match key_type {
			0 => GraphState::generate_keypair(GraphKeyType::X25519)
				.map_err(|e| SdkJniError::from(e))
				.and_then(|key_pair| serialize_graph_keypair(&env, &key_pair)),
			_ => Err(SdkJniError::BadJniParameter("invalid graph_key_type")),
		}
	});
	handle_result(&mut env, result)
}
