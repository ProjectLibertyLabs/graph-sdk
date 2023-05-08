use crate::bindings::*;
use dsnp_graph_config::{ConnectionType, DsnpVersion, PrivacyType};
use std::collections::HashMap;

// Function to convert C-compatible `SchemaConfig` to a Rust `SchemaConfig`
pub fn schema_config_from_ffi(schema_config: &SchemaConfig) -> dsnp_graph_config::SchemaConfig {
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
pub fn config_from_ffi(config: &Config) -> dsnp_graph_config::Config {
	let schema_map_slice =
		unsafe { std::slice::from_raw_parts(config.schema_map, config.schema_map_len) };
	let mut schema_map = HashMap::new();
	for schema_config_tuple in schema_map_slice {
		let id = schema_config_tuple.schema_id;
		let schema_config = &schema_config_tuple.schema_config;
		schema_map.insert(id, schema_config_from_ffi(schema_config));
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
		sdk_max_stale_friendship_days: config.sdk_max_stale_friendship_days,
		max_graph_page_size_bytes: config.max_graph_page_size_bytes,
		max_page_id: config.max_page_id,
		max_key_page_size_bytes: config.max_key_page_size_bytes,
		schema_map,
		dsnp_versions,
	}
}

// Function to convert C-compatible `SchemaConfig` to a Rust `SchemaConfig`
pub fn environment_from_ffi(environment: &Environment) -> dsnp_graph_config::Environment {
	let config = config_from_ffi(&environment.config);
	match environment.environment_type {
		EnvironmentType::Mainnet => dsnp_graph_config::Environment::Mainnet,
		EnvironmentType::Rococo => dsnp_graph_config::Environment::Rococo,
		EnvironmentType::Dev => dsnp_graph_config::Environment::Dev(config),
	}
}

// Function to convert C-compatible `GraphKeyPair` to a Rust `GraphKeyPair`
fn graph_key_pair_from_ffi(
	graph_key_pair: &GraphKeyPair,
) -> dsnp_graph_core::dsnp::api_types::GraphKeyPair {
	let public_key = unsafe {
		std::slice::from_raw_parts(graph_key_pair.public_key, graph_key_pair.public_key_len)
	};
	let secret_key = unsafe {
		std::slice::from_raw_parts(graph_key_pair.secret_key, graph_key_pair.secret_key_len)
	};
	dsnp_graph_core::dsnp::api_types::GraphKeyPair {
		key_type: graph_key_pair.key_type.clone(),
		public_key: public_key.to_vec(),
		secret_key: secret_key.to_vec(),
	}
}

fn key_data_from_ffi(key_data: &KeyData) -> dsnp_graph_core::dsnp::api_types::KeyData {
	let content = unsafe { std::slice::from_raw_parts(key_data.content, key_data.content_len) };
	dsnp_graph_core::dsnp::api_types::KeyData { index: key_data.index, content: content.to_vec() }
}

fn dsnp_keys_from_ffi(dsnp_keys: &DsnpKeys) -> dsnp_graph_core::dsnp::api_types::DsnpKeys {
	let keys = unsafe { std::slice::from_raw_parts(dsnp_keys.keys, dsnp_keys.keys_len) };
	let key_data = keys.iter().map(|key| key_data_from_ffi(key)).collect();

	dsnp_graph_core::dsnp::api_types::DsnpKeys {
		dsnp_user_id: dsnp_keys.dsnp_user_id.clone(),
		keys_hash: dsnp_keys.keys_hash.clone(),
		keys: key_data,
	}
}

// Function to convert C-compatible PageData to Rust PageData
fn page_data_from_ffi(page_data: &PageData) -> dsnp_graph_core::dsnp::api_types::PageData {
	let content = unsafe { std::slice::from_raw_parts(page_data.content, page_data.content_len) };
	dsnp_graph_core::dsnp::api_types::PageData {
		page_id: page_data.page_id,
		content: content.to_vec(),
		content_hash: page_data.content_hash,
	}
}

// Function to convert C-compatible ImportBundle to Rust ImportBundle
pub fn import_bundle_from_ffi(
	import_bundle: &ImportBundle,
) -> dsnp_graph_core::dsnp::api_types::ImportBundle {
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

	dsnp_graph_core::dsnp::api_types::ImportBundle {
		dsnp_user_id: import_bundle.dsnp_user_id.clone(),
		schema_id: import_bundle.schema_id,
		key_pairs,
		dsnp_keys,
		pages,
	}
}

// Function to convert C-compatible `ImportBundle` slice to a Rust `ImportBundle` vector
pub fn payloads_from_ffi(
	payloads: &[ImportBundle],
) -> Vec<dsnp_graph_core::dsnp::api_types::ImportBundle> {
	let mut rust_payloads = Vec::new();
	for payload in payloads {
		let rust_payload = import_bundle_from_ffi(payload);
		rust_payloads.push(rust_payload);
	}
	rust_payloads
}

// Function to convert C-compatible `Update` to a Rust `Update`
pub fn updates_to_ffi(updates: Vec<dsnp_graph_core::dsnp::api_types::Update>) -> Vec<Update> {
	let mut ffi_updates = Vec::new();
	for update in updates {
		match update {
			dsnp_graph_core::dsnp::api_types::Update::PersistPage {
				owner_dsnp_user_id,
				schema_id,
				page_id,
				prev_hash,
				mut payload,
			} => {
				let ffi_persist_page = PersistPage {
					owner_dsnp_user_id: owner_dsnp_user_id.clone(),
					schema_id,
					page_id,
					prev_hash,
					payload: payload.as_mut_ptr(),
					payload_len: payload.len(),
				};
				ffi_updates.push(Update::Persist(ffi_persist_page));
			},
			dsnp_graph_core::dsnp::api_types::Update::DeletePage {
				owner_dsnp_user_id,
				schema_id,
				page_id,
				prev_hash,
			} => {
				let ffi_delete_page = DeletePage {
					owner_dsnp_user_id: owner_dsnp_user_id.clone(),
					schema_id,
					page_id,
					prev_hash,
				};
				ffi_updates.push(Update::Delete(ffi_delete_page));
			},
			dsnp_graph_core::dsnp::api_types::Update::AddKey {
				owner_dsnp_user_id,
				prev_hash,
				mut payload,
			} => {
				let ffi_add_key = AddKey {
					owner_dsnp_user_id: owner_dsnp_user_id.clone(),
					prev_hash,
					payload: payload.as_mut_ptr(),
					payload_len: payload.len(),
				};
				ffi_updates.push(Update::Add(ffi_add_key));
			},
		}
	}
	ffi_updates
}
