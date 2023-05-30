use crate::mappings::{
	convert_jboolean, map_to_actions, map_to_environment, map_to_imports, serialize_config,
	serialize_dsnp_users, serialize_graph_edges, serialize_graph_updates, serialize_public_keys,
};
use dsnp_graph_config::{DsnpUserId, SchemaId};
use dsnp_graph_core::api::api::{GraphAPI, GraphState};
use jni::{
	objects::{JByteArray, JClass, JObject, JString},
	sys::{jboolean, jint, jlong, JNI_ERR, JNI_FALSE},
	JNIEnv,
};
use std::{
	ops::{Deref, DerefMut},
	panic,
	sync::Mutex,
};

// Collection of GraphStates
static GRAPH_STATES: Mutex<Vec<jlong>> = Mutex::new(Vec::new());

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

#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_initializeGraphState<'local>(
	env: JNIEnv<'local>,
	_class: JClass<'local>,
	environment: JByteArray,
) -> jlong {
	let result = panic::catch_unwind(|| {
		let rust_environment = map_to_environment(&env, &environment).unwrap();
		let graph_state = GraphState::new(rust_environment);
		let boxed = Box::new(graph_state);
		let handle = Box::into_raw(boxed) as jlong;
		let mut graph_states = GRAPH_STATES.lock().unwrap();
		graph_states.push(handle);
		handle
	});
	match result {
		Ok(handle) => handle,
		Err(_) => 0 as jlong,
	}
}

#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_freeGraphState<'local>(
	mut _env: JNIEnv<'local>,
	_class: JClass<'local>,
	handle: jlong,
) {
	let result = panic::catch_unwind(|| {
		if handle == 0 {
			return
		}
		let mut graph_states = GRAPH_STATES.lock().unwrap();
		let index = graph_states.iter().position(|x| *x == handle).unwrap();
		graph_states.remove(index);
		let _ = unsafe { Box::from_raw(handle as *mut GraphState) };
	});
	result.unwrap_or(())
}

#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_getConfig<'local>(
	env: JNIEnv<'local>,
	_class: JClass<'local>,
	environment: JByteArray,
) -> JByteArray<'local> {
	let result = panic::catch_unwind(|| {
		let rust_environment = map_to_environment(&env, &environment).unwrap();
		let config = rust_environment.get_config();
		let result = serialize_config(&env, &config).unwrap();
		result
	});
	result.unwrap_or(JByteArray::default())
}

#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_containsUserGraph<'local>(
	mut _env: JNIEnv<'local>,
	_class: JClass<'local>,
	handle: jlong,
	dsnp_user_id: jlong,
) -> jboolean {
	let result = panic::catch_unwind(|| {
		if handle == 0 {
			return JNI_FALSE
		}
		let graph_states = GRAPH_STATES.lock().unwrap();
		if !graph_states.contains(&handle) {
			return JNI_FALSE
		}
		let graph = unsafe { Box::from_raw(handle as *mut GraphState) };
		// TODO: test edge case that dsnp_user_id is bigger than i64
		let user_id = u64::try_from(dsnp_user_id).unwrap();
		let result = graph.deref().contains_user_graph(&user_id).into();
		// pulling out of the box as raw so that memory stays allocated
		let _ = Box::into_raw(graph) as jlong;
		result
	});
	result.unwrap_or(JNI_FALSE)
}

#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_getGraphUsersLength<'local>(
	mut _env: JNIEnv<'local>,
	_class: JClass<'local>,
	handle: jlong,
) -> jint {
	let result = panic::catch_unwind(|| {
		if handle == 0 {
			return JNI_ERR
		}
		let graph_states = GRAPH_STATES.lock().unwrap();
		if !graph_states.contains(&handle) {
			return JNI_ERR
		}
		let graph = unsafe { Box::from_raw(handle as *mut GraphState) };
		let result = graph.deref().len() as jint;
		// pulling out of the box as raw so that memory stays allocated
		let _ = Box::into_raw(graph) as jlong;
		result
	});
	result.unwrap_or(JNI_ERR)
}

#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_removeUserGraph<'local>(
	mut _env: JNIEnv<'local>,
	_class: JClass<'local>,
	handle: jlong,
	dsnp_user_id: jlong,
) {
	let _ = panic::catch_unwind(|| {
		if handle == 0 {
			return
		}
		let graph_states = GRAPH_STATES.lock().unwrap();
		if !graph_states.contains(&handle) {
			return
		}
		let mut graph = unsafe { Box::from_raw(handle as *mut GraphState) };
		let user_id = u64::try_from(dsnp_user_id).unwrap();
		graph.deref_mut().remove_user_graph(&user_id);
		// pulling out of the box as raw so that memory stays allocated
		let _ = Box::into_raw(graph) as jlong;
	});
}

#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_importUserData<'local>(
	env: JNIEnv<'local>,
	_class: JClass<'local>,
	handle: jlong,
	imports: JByteArray,
) {
	let _ = panic::catch_unwind(|| {
		if handle == 0 {
			return
		}
		let graph_states = GRAPH_STATES.lock().unwrap();
		if !graph_states.contains(&handle) {
			return
		}
		let mut graph = unsafe { Box::from_raw(handle as *mut GraphState) };
		let rust_imports = map_to_imports(&env, &imports).unwrap();
		graph.deref_mut().import_users_data(&rust_imports).unwrap();
		// pulling out of the box as raw so that memory stays allocated
		let _ = Box::into_raw(graph) as jlong;
	});
}

#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_exportUpdates<'local>(
	env: JNIEnv<'local>,
	_class: JClass<'local>,
	handle: jlong,
) -> JByteArray<'local> {
	let result = panic::catch_unwind(|| {
		if handle == 0 {
			return JByteArray::default()
		}
		let graph_states = GRAPH_STATES.lock().unwrap();
		if !graph_states.contains(&handle) {
			return JByteArray::default()
		}
		let graph = unsafe { Box::from_raw(handle as *mut GraphState) };
		let updates = graph.deref().export_updates().unwrap();
		let result = serialize_graph_updates(&env, &updates).unwrap();
		// pulling out of the box as raw so that memory stays allocated
		let _ = Box::into_raw(graph) as jlong;
		result
	});
	result.unwrap_or(JByteArray::default())
}

#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_applyActions<'local>(
	env: JNIEnv<'local>,
	_class: JClass<'local>,
	handle: jlong,
	actions: JByteArray,
) {
	let _ = panic::catch_unwind(|| {
		if handle == 0 {
			return
		}
		let actions = map_to_actions(&env, &actions).unwrap();
		let graph_states = GRAPH_STATES.lock().unwrap();
		if !graph_states.contains(&handle) {
			return
		}
		let mut graph = unsafe { Box::from_raw(handle as *mut GraphState) };
		graph.deref_mut().apply_actions(&actions).unwrap();
		// pulling out of the box as raw so that memory stays allocated
		let _ = Box::into_raw(graph) as jlong;
	});
}

#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_forceCalculateGraphs<'local>(
	env: JNIEnv<'local>,
	_class: JClass<'local>,
	handle: jlong,
	dsnp_user_id: jlong,
) -> JByteArray<'local> {
	let result = panic::catch_unwind(|| {
		if handle == 0 {
			return JByteArray::default()
		}
		let graph_states = GRAPH_STATES.lock().unwrap();
		if !graph_states.contains(&handle) {
			return JByteArray::default()
		}
		let graph = unsafe { Box::from_raw(handle as *mut GraphState) };
		let dsnp_user_id = DsnpUserId::try_from(dsnp_user_id).unwrap();
		let updates = graph.deref().force_recalculate_graphs(&dsnp_user_id).unwrap();
		let result = serialize_graph_updates(&env, &updates).unwrap();
		// pulling out of the box as raw so that memory stays allocated
		let _ = Box::into_raw(graph) as jlong;
		result
	});
	result.unwrap_or(JByteArray::default())
}

#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_getConnectionsForUserGraph<'local>(
	env: JNIEnv<'local>,
	_class: JClass<'local>,
	handle: jlong,
	dsnp_user_id: jlong,
	schema_id: jint,
	include_pending: jboolean,
) -> JByteArray<'local> {
	let result = panic::catch_unwind(|| {
		if handle == 0 {
			return JByteArray::default()
		}
		let graph_states = GRAPH_STATES.lock().unwrap();
		if !graph_states.contains(&handle) {
			return JByteArray::default()
		}
		let graph = unsafe { Box::from_raw(handle as *mut GraphState) };
		let dsnp_user_id = DsnpUserId::try_from(dsnp_user_id).unwrap();
		let schema_id = SchemaId::try_from(schema_id).unwrap();
		let include_pending = convert_jboolean(include_pending).unwrap();
		let graph_edges = graph
			.deref()
			.get_connections_for_user_graph(&dsnp_user_id, &schema_id, include_pending)
			.unwrap();
		let result = serialize_graph_edges(&env, &graph_edges).unwrap();
		// pulling out of the box as raw so that memory stays allocated
		let _ = Box::into_raw(graph) as jlong;
		result
	});
	result.unwrap_or(JByteArray::default())
}

#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_getUsersWithoutKeys<'local>(
	env: JNIEnv<'local>,
	_class: JClass<'local>,
	handle: jlong,
) -> JByteArray<'local> {
	let result = panic::catch_unwind(|| {
		if handle == 0 {
			return JByteArray::default()
		}
		let graph_states = GRAPH_STATES.lock().unwrap();
		if !graph_states.contains(&handle) {
			return JByteArray::default()
		}
		let graph = unsafe { Box::from_raw(handle as *mut GraphState) };
		let dsnp_users = graph.deref().get_connections_without_keys().unwrap();
		let result = serialize_dsnp_users(&env, &dsnp_users).unwrap();
		// pulling out of the box as raw so that memory stays allocated
		let _ = Box::into_raw(graph) as jlong;
		result
	});
	result.unwrap_or(JByteArray::default())
}

#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_getOneSidedPrivateFriendshipConnections<
	'local,
>(
	env: JNIEnv<'local>,
	_class: JClass<'local>,
	handle: jlong,
	dsnp_user_id: jlong,
) -> JByteArray<'local> {
	let result = panic::catch_unwind(|| {
		if handle == 0 {
			return JByteArray::default()
		}
		let graph_states = GRAPH_STATES.lock().unwrap();
		if !graph_states.contains(&handle) {
			return JByteArray::default()
		}
		let graph = unsafe { Box::from_raw(handle as *mut GraphState) };
		let user_id = u64::try_from(dsnp_user_id).unwrap();
		let graph_edges =
			graph.deref().get_one_sided_private_friendship_connections(&user_id).unwrap();
		let result = serialize_graph_edges(&env, &graph_edges).unwrap();
		// pulling out of the box as raw so that memory stays allocated
		let _ = Box::into_raw(graph) as jlong;
		result
	});
	result.unwrap_or(JByteArray::default())
}

#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_getPublicKeys<'local>(
	env: JNIEnv<'local>,
	_class: JClass<'local>,
	handle: jlong,
	dsnp_user_id: jlong,
) -> JByteArray<'local> {
	let result = panic::catch_unwind(|| {
		if handle == 0 {
			return JByteArray::default()
		}
		let graph_states = GRAPH_STATES.lock().unwrap();
		if !graph_states.contains(&handle) {
			return JByteArray::default()
		}
		let graph = unsafe { Box::from_raw(handle as *mut GraphState) };
		let user_id = u64::try_from(dsnp_user_id).unwrap();
		let public_keys = graph.deref().get_public_keys(&user_id).unwrap();
		let result = serialize_public_keys(&env, &public_keys).unwrap();
		// pulling out of the box as raw so that memory stays allocated
		let _ = Box::into_raw(graph) as jlong;
		result
	});
	result.unwrap_or(JByteArray::default())
}
