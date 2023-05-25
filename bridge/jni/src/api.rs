use crate::mappings::map_to_environment;
use dsnp_graph_core::api::api::GraphState;
use jni::{
	objects::{JByteArray, JClass, JObject, JString},
	sys::{jboolean, jint, jlong},
	JNIEnv,
};
use std::{panic, sync::Mutex};

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
	mut _env: JNIEnv<'local>,
	_class: JClass<'local>,
	_environment: JByteArray,
) -> JByteArray<'local> {
	unimplemented!("Java_io_amplica_graphsdk_Native_getConfig")
}

#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_containsUserGraph<'local>(
	mut _env: JNIEnv<'local>,
	_class: JClass<'local>,
	_handle: jlong,
	_dsnp_user_id: jlong,
) -> jboolean {
	unimplemented!("Java_io_amplica_graphsdk_Native_containsUserGraph")
}

#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_getGraphUsersLength<'local>(
	mut _env: JNIEnv<'local>,
	_class: JClass<'local>,
	_handle: jlong,
) -> jint {
	unimplemented!("Java_io_amplica_graphsdk_Native_getGraphUsersLength")
}

#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_removeUserGraph<'local>(
	mut _env: JNIEnv<'local>,
	_class: JClass<'local>,
	_handle: jlong,
	_dsnp_user_id: jlong,
) {
	unimplemented!("Java_io_amplica_graphsdk_Native_removeUserGraph")
}

#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_importUserData<'local>(
	mut _env: JNIEnv<'local>,
	_class: JClass<'local>,
	_handle: jlong,
	_import: JByteArray,
) -> JByteArray<'local> {
	unimplemented!("Java_io_amplica_graphsdk_Native_importUserData")
}

#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_exportUpdates<'local>(
	mut _env: JNIEnv<'local>,
	_class: JClass<'local>,
	_handle: jlong,
) -> JByteArray<'local> {
	unimplemented!("Java_io_amplica_graphsdk_Native_exportUpdates")
}

#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_applyActions<'local>(
	mut _env: JNIEnv<'local>,
	_class: JClass<'local>,
	_handle: jlong,
	_actions: JByteArray,
) {
	unimplemented!("Java_io_amplica_graphsdk_Native_applyActions")
}

#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_forceCalculateGraphs<'local>(
	mut _env: JNIEnv<'local>,
	_class: JClass<'local>,
	_handle: jlong,
	_dsnp_user_id: jlong,
) -> JByteArray<'local> {
	unimplemented!("Java_io_amplica_graphsdk_Native_forceCalculateGraphs")
}

#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_getConnectionsForUserGraph<'local>(
	mut _env: JNIEnv<'local>,
	_class: JClass<'local>,
	_handle: jlong,
	_dsnp_user_id: jlong,
	_schema_id: jint,
	_include_pending: jboolean,
) -> JByteArray<'local> {
	unimplemented!("Java_io_amplica_graphsdk_Native_getConnectionsForUserGraph")
}

#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_getUsersWithoutKeys<'local>(
	mut _env: JNIEnv<'local>,
	_class: JClass<'local>,
	_handle: jlong,
) -> JByteArray<'local> {
	unimplemented!("Java_io_amplica_graphsdk_Native_getUsersWithoutKeys")
}

#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_getOneSidedPrivateFriendshipConnections<
	'local,
>(
	mut _env: JNIEnv<'local>,
	_class: JClass<'local>,
	_handle: jlong,
	_dsnp_user_id: jlong,
) -> JByteArray<'local> {
	unimplemented!("Java_io_amplica_graphsdk_Native_getConnectionsForUserGraph")
}

#[no_mangle]
pub unsafe extern "C" fn Java_io_amplica_graphsdk_Native_getPublicKeys<'local>(
	mut _env: JNIEnv<'local>,
	_class: JClass<'local>,
	_handle: jlong,
	_dsnp_user_id: jlong,
) -> JByteArray<'local> {
	unimplemented!("Java_io_amplica_graphsdk_Native_getConnectionsForUserGraph")
}
