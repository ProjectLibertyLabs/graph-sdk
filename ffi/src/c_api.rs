use crate::bindings::*;
use dsnp_graph_core::api::api::GraphState as GraphStateRust;
use libc::{c_int, c_void, size_t};
use std::collections::HashMap;

// Define a C-compatible representation of GraphState
pub struct GraphState {
	inner: *mut c_void,
}

// Define a C-compatible representation of GraphAPI
#[repr(C)]
pub struct GraphAPI {
	graph_state_new: extern "C" fn(environment: *const Environment) -> *mut GraphState,
	graph_state_free: extern "C" fn(*mut GraphState),
	graph_state_with_capacity: extern "C" fn(*const Environment, size_t) -> *mut GraphState,
	graph_state_get_capacity: extern "C" fn(*const GraphState) -> size_t,
	// Use raw pointers instead of references, since references are not C-compatible
	contains_user_graph: extern "C" fn(*const GraphState, *const u64) -> bool,
	len: extern "C" fn(*const GraphState) -> usize,
	remove_user_graph: extern "C" fn(*const GraphState, *const u64),
	import_users_data: extern "C" fn(*const GraphState, *const ImportBundle, size_t) -> c_int,
	export_updates: extern "C" fn(*const GraphState, *mut *mut Update, *mut size_t) -> c_int,
	apply_actions: extern "C" fn(*const GraphState, *const Action, size_t) -> c_int,
	get_connections_for_user_graph: extern "C" fn(
		*const GraphState,
		*const u64,
		*const u16,
		bool,
		*mut *mut DsnpGraphEdge,
		*mut size_t,
	) -> c_int,
	get_connections_without_keys:
		extern "C" fn(*const GraphState, *mut *mut u64, *mut size_t) -> c_int,
}

// Function to convert C-compatible `SchemaConfig` to a Rust `SchemaConfig`
fn schema_config_from_ffi(schema_config: &SchemaConfig) -> dsnp_graph_config::SchemaConfig {
	let dsnp_version = match schema_config.dsnp_version {
		DsnpVersion::Version1_0 => dsnp_graph_config::DsnpVersion::Version1_0,
	};

	let connection_type = match &schema_config.connection_type {
		ConnectionType::Follow(privacy_type) => {
			let privacy = match privacy_type {
				PrivacyType::Public => dsnp_graph_config::PrivacyType::Public,
				PrivacyType::Private => dsnp_graph_config::PrivacyType::Private,
			};
			dsnp_graph_config::ConnectionType::Follow(privacy)
		},
		ConnectionType::Friendship(privacy_type) => {
			let privacy = match privacy_type {
				PrivacyType::Public => dsnp_graph_config::PrivacyType::Public,
				PrivacyType::Private => dsnp_graph_config::PrivacyType::Private,
			};
			dsnp_graph_config::ConnectionType::Friendship(privacy)
		},
	};

	dsnp_graph_config::SchemaConfig { dsnp_version, connection_type }
}

// Function to convert C-compatible `Config` to a Rust `Config`
fn config_from_ffi(config: &Config) -> dsnp_graph_config::Config {
	let schema_map_slice =
		unsafe { std::slice::from_raw_parts(config.schema_map, config.schema_map_len) };
	let mut schema_map = HashMap::new();
	for (id, schema_config) in schema_map_slice {
		schema_map.insert(*id, schema_config_from_ffi(schema_config));
	}

	let dsnp_versions_slice =
		unsafe { std::slice::from_raw_parts(config.dsnp_versions, config.dsnp_versions_len) };
	let mut dsnp_versions = Vec::new();
	for version in dsnp_versions_slice {
		let rust_version = match version {
			DsnpVersion::Version1_0 => dsnp_graph_config::DsnpVersion::Version1_0,
		};
		dsnp_versions.push(rust_version);
	}
	dsnp_graph_config::Config {
		sdk_max_users_graph_size: config.sdk_max_users_graph_size,
		max_graph_page_size_bytes: config.max_graph_page_size_bytes,
		max_page_id: config.max_page_id,
		max_key_page_size_bytes: config.max_key_page_size_bytes,
		schema_map,
		dsnp_versions,
	}
}

// Function to convert C-compatible `SchemaConfig` to a Rust `SchemaConfig`
fn environment_from_ffi(environment: &Environment) -> dsnp_graph_config::Environment {
	let config = config_from_ffi(&environment.config);
	match environment.environment_type {
		EnvironmentType::Mainnet => dsnp_graph_config::Environment::Mainnet,
		EnvironmentType::Rococo => dsnp_graph_config::Environment::Rococo,
		EnvironmentType::Dev => dsnp_graph_config::Environment::Dev(config),
	}
}

// Implement graph_state_new function
#[no_mangle]
pub unsafe extern "C" fn graph_state_new(environment: *const Environment) -> *mut GraphState {
	let environment = &*environment;
	let rust_environment = environment_from_ffi(environment);
	let graph_state = GraphStateRust::new(rust_environment);
	let c_graph_state = GraphState { inner: Box::into_raw(Box::new(graph_state)) as *mut c_void };
	Box::into_raw(Box::new(c_graph_state))
}

// Implement graph_state_free function
#[no_mangle]
pub unsafe extern "C" fn graph_state_free(graph_state: *mut GraphState) {
	if graph_state.is_null() {
		return
	}
	let graph_state = Box::from_raw(graph_state);
	let _ = Box::from_raw(graph_state.inner as *mut GraphStateRust);
}
