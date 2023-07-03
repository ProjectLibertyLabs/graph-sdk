use crate::bindings::*;
use dsnp_graph_config::{Config as RustConfig, DsnpVersion, MAINNET_CONFIG, ROCOCO_CONFIG};
use std::{collections::HashMap, mem::ManuallyDrop};

pub fn get_config_for_ffi(environment: &Environment) -> Config {
	match environment {
		Environment::Mainnet => get_config_from_rust_config(&MAINNET_CONFIG),
		Environment::Rococo => get_config_from_rust_config(&ROCOCO_CONFIG),
		Environment::Dev(config) => config.clone(),
	}
}

fn get_config_from_rust_config(rust_config: &RustConfig) -> Config {
	let schema_map = rust_config
		.schema_map
		.iter()
		.map(|(schema_id, schema_config)| SchemaConfigTuple {
			schema_id: *schema_id,
			schema_config: schema_config.clone(),
		})
		.collect::<Vec<SchemaConfigTuple>>();

	let dsnp_versions = rust_config
		.dsnp_versions
		.iter()
		.map(|version| match version {
			dsnp_graph_config::DsnpVersion::Version1_0 => DsnpVersion::Version1_0,
		})
		.collect::<Vec<DsnpVersion>>();

	Config {
		sdk_max_stale_friendship_days: rust_config.sdk_max_stale_friendship_days,
		max_graph_page_size_bytes: rust_config.max_graph_page_size_bytes,
		max_page_id: rust_config.max_page_id,
		max_key_page_size_bytes: rust_config.max_key_page_size_bytes,
		graph_key_pair_schema_id: rust_config.graph_key_pair_schema_id,
		schema_map_len: schema_map.len(),
		schema_map: ManuallyDrop::new(schema_map).as_mut_ptr(),
		dsnp_versions_len: dsnp_versions.len(),
		dsnp_versions: ManuallyDrop::new(dsnp_versions).as_mut_ptr(),
	}
}

// Function to convert C-compatible `Config` to a Rust `Config`
pub fn config_from_ffi(config: &Config) -> dsnp_graph_config::Config {
	let schema_map_slice = if config.schema_map.is_null() {
		&[]
	} else {
		unsafe { std::slice::from_raw_parts(config.schema_map, config.schema_map_len) }
	};
	let mut schema_map = HashMap::new();
	for schema_config_tuple in schema_map_slice {
		schema_map.insert(schema_config_tuple.schema_id, schema_config_tuple.schema_config.clone());
	}

	let dsnp_versions_slice = if config.dsnp_versions.is_null() {
		&[]
	} else {
		unsafe { std::slice::from_raw_parts(config.dsnp_versions, config.dsnp_versions_len) }
	};

	let mut dsnp_versions = Vec::new();
	for version in dsnp_versions_slice {
		let rust_version = match version {
			DsnpVersion::Version1_0 => dsnp_graph_config::DsnpVersion::Version1_0,
		};
		dsnp_versions.push(rust_version);
	}
	dsnp_graph_config::Config {
		sdk_max_stale_friendship_days: config.sdk_max_stale_friendship_days,
		max_graph_page_size_bytes: config.max_graph_page_size_bytes,
		max_page_id: config.max_page_id,
		max_key_page_size_bytes: config.max_key_page_size_bytes,
		graph_key_pair_schema_id: config.graph_key_pair_schema_id,
		schema_map,
		dsnp_versions,
	}
}

// Function to convert C-compatible `SchemaConfig` to a Rust `SchemaConfig`
pub fn environment_from_ffi(environment: &Environment) -> dsnp_graph_config::Environment {
	match environment {
		Environment::Mainnet => dsnp_graph_config::Environment::Mainnet,
		Environment::Rococo => dsnp_graph_config::Environment::Rococo,
		Environment::Dev(config) => {
			let rust_config = config_from_ffi(config);
			dsnp_graph_config::Environment::Dev(rust_config)
		},
	}
}

// Function to convert C-compatible `GraphKeyPair` to a Rust `GraphKeyPair`
fn graph_key_pair_from_ffi(
	graph_key_pair: &GraphKeyPair,
) -> dsnp_graph_core::api::api_types::GraphKeyPair {
	let public_key = unsafe {
		std::slice::from_raw_parts(graph_key_pair.public_key, graph_key_pair.public_key_len)
	};
	let secret_key = unsafe {
		std::slice::from_raw_parts(graph_key_pair.secret_key, graph_key_pair.secret_key_len)
	};
	dsnp_graph_core::api::api_types::GraphKeyPair {
		key_type: graph_key_pair.key_type.clone(),
		public_key: public_key.to_vec(),
		secret_key: secret_key.to_vec(),
	}
}

fn key_data_from_ffi(key_data: &KeyData) -> dsnp_graph_core::api::api_types::KeyData {
	let content = unsafe { std::slice::from_raw_parts(key_data.content, key_data.content_len) };
	dsnp_graph_core::api::api_types::KeyData { index: key_data.index, content: content.to_vec() }
}

pub fn dsnp_keys_from_ffi(dsnp_keys: &DsnpKeys) -> dsnp_graph_core::api::api_types::DsnpKeys {
	let keys = unsafe { std::slice::from_raw_parts(dsnp_keys.keys, dsnp_keys.keys_len) };
	let key_data = keys.iter().map(|key| key_data_from_ffi(key)).collect();

	dsnp_graph_core::api::api_types::DsnpKeys {
		dsnp_user_id: dsnp_keys.dsnp_user_id,
		keys_hash: dsnp_keys.keys_hash,
		keys: key_data,
	}
}

// Function to convert C-compatible PageData to Rust PageData
fn page_data_from_ffi(page_data: &PageData) -> dsnp_graph_core::api::api_types::PageData {
	let content = unsafe { std::slice::from_raw_parts(page_data.content, page_data.content_len) };
	dsnp_graph_core::api::api_types::PageData {
		page_id: page_data.page_id,
		content: content.to_vec(),
		content_hash: page_data.content_hash,
	}
}

// Function to convert C-compatible ImportBundle to Rust ImportBundle
pub fn import_bundle_from_ffi(
	import_bundle: &ImportBundle,
) -> dsnp_graph_core::api::api_types::ImportBundle {
	let key_pairs_slice =
		unsafe { std::slice::from_raw_parts(import_bundle.key_pairs, import_bundle.key_pairs_len) };
	let mut key_pairs = Vec::new();
	for graph_key_pair in key_pairs_slice {
		key_pairs.push(graph_key_pair_from_ffi(graph_key_pair));
	}

	let dsnp_keys = dsnp_keys_from_ffi(&import_bundle.dsnp_keys);

	let pages_slice =
		unsafe { std::slice::from_raw_parts(import_bundle.pages, import_bundle.pages_len) };
	let mut pages = Vec::new();
	for page_data in pages_slice {
		pages.push(page_data_from_ffi(page_data));
	}

	dsnp_graph_core::api::api_types::ImportBundle {
		dsnp_user_id: import_bundle.dsnp_user_id,
		schema_id: import_bundle.schema_id,
		key_pairs,
		dsnp_keys,
		pages,
	}
}

// Function to convert C-compatible `ImportBundle` slice to a Rust `ImportBundle` vector
pub fn payloads_from_ffi(
	payloads: &[ImportBundle],
) -> Vec<dsnp_graph_core::api::api_types::ImportBundle> {
	let mut rust_payloads = Vec::new();
	for payload in payloads {
		let rust_payload = import_bundle_from_ffi(payload);
		rust_payloads.push(rust_payload);
	}
	rust_payloads
}

// Function to convert Rust `Update` to C-compatible `Update`
pub fn updates_to_ffi(updates: Vec<dsnp_graph_core::api::api_types::Update>) -> Vec<Update> {
	let mut ffi_updates = Vec::new();
	for update in updates {
		match update {
			dsnp_graph_core::api::api_types::Update::PersistPage {
				owner_dsnp_user_id,
				schema_id,
				page_id,
				prev_hash,
				payload,
			} => {
				let ffi_persist_page = PersistPage {
					owner_dsnp_user_id,
					schema_id,
					page_id,
					prev_hash,
					payload_len: payload.len(),
					payload: ManuallyDrop::new(payload).as_mut_ptr(),
				};
				ffi_updates.push(Update::Persist(ffi_persist_page));
			},
			dsnp_graph_core::api::api_types::Update::DeletePage {
				owner_dsnp_user_id,
				schema_id,
				page_id,
				prev_hash,
			} => {
				let ffi_delete_page =
					DeletePage { owner_dsnp_user_id, schema_id, page_id, prev_hash };
				ffi_updates.push(Update::Delete(ffi_delete_page));
			},
			dsnp_graph_core::api::api_types::Update::AddKey {
				owner_dsnp_user_id,
				prev_hash,
				payload,
			} => {
				let ffi_add_key = AddKey {
					owner_dsnp_user_id,
					prev_hash,
					payload_len: payload.len(),
					payload: ManuallyDrop::new(payload).as_mut_ptr(),
				};
				ffi_updates.push(Update::Add(ffi_add_key));
			},
		}
	}
	ffi_updates
}

pub fn actions_from_ffi(actions: &[Action]) -> Vec<dsnp_graph_core::api::api_types::Action> {
	let mut rust_actions = Vec::new();
	for action in actions {
		match action {
			Action::Connect { owner_dsnp_user_id, connection, dsnp_keys } => {
				let rust_action = dsnp_graph_core::api::api_types::Action::Connect {
					owner_dsnp_user_id: *owner_dsnp_user_id,
					connection: connection.clone(),
					dsnp_keys: match unsafe { dsnp_keys.as_ref() } {
						Some(keys) => Some(dsnp_keys_from_ffi(keys)),
						None => None,
					},
				};
				rust_actions.push(rust_action);
			},
			Action::Disconnect { owner_dsnp_user_id, connection } => {
				let rust_action = dsnp_graph_core::api::api_types::Action::Disconnect {
					owner_dsnp_user_id: *owner_dsnp_user_id,
					connection: connection.clone(),
				};
				rust_actions.push(rust_action);
			},
			Action::AddGraphKey { owner_dsnp_user_id, new_public_key, new_public_key_len } => {
				let new_public_key =
					unsafe { std::slice::from_raw_parts(*new_public_key, *new_public_key_len) };
				let rust_action = dsnp_graph_core::api::api_types::Action::AddGraphKey {
					owner_dsnp_user_id: *owner_dsnp_user_id,
					new_public_key: new_public_key.to_vec(),
				};
				rust_actions.push(rust_action);
			},
		}
	}
	rust_actions
}

pub fn dsnp_public_keys_to_ffi(
	keys: Vec<dsnp_graph_core::dsnp::dsnp_types::DsnpPublicKey>,
) -> Vec<DsnpPublicKey> {
	keys.into_iter()
		.map(|key| DsnpPublicKey {
			key_id: key.key_id.unwrap(), // unwrap is fine since it should be always populated
			content_len: key.key.len(),
			content: ManuallyDrop::new(key.key).as_mut_ptr(),
		})
		.collect()
}
