//! Helper functions for converting between JS and Rust types and vice versa
use dsnp_graph_config::{
	Config, ConnectionType, DsnpUserId, DsnpVersion, Environment, PageId, SchemaConfig, SchemaId,
};
use dsnp_graph_core::{
	api::api_types::{
		Action, Connection, DsnpKeys, GraphKeyPair, ImportBundle, KeyData, PageData, PageHash,
		Update,
	},
	dsnp::dsnp_types::{DsnpGraphEdge, DsnpPublicKey},
};
use neon::{
	handle::Handle,
	object::Object,
	prelude::{Context, FunctionContext},
	result::{JsResult, NeonResult},
	types::{buffer::TypedArray, JsArray, JsNumber, JsObject, JsString, JsTypedArray},
};

/// Convert environment from JSObject to Environment
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `environment_from_js` - Neon JsObject containing the environment
/// # Returns
/// * `JsResult<Environment>` - Neon Environment object
/// # Errors
/// * Throws a Neon error if the environment cannot be converted
/// # Safety
pub unsafe fn environment_from_js(
	cx: &mut FunctionContext,
	environment_from_js: Handle<JsObject>,
) -> NeonResult<Environment> {
	let environment_type_str: Handle<JsString> =
		environment_from_js.get(cx, "environmentType").unwrap_or(cx.string(""));

	match environment_type_str.value(cx).as_str() {
		"Mainnet" => Ok(Environment::Mainnet),
		"Rococo" => Ok(Environment::Rococo),
		"Dev" => {
			let config: Handle<JsObject> = environment_from_js.get(cx, "config").unwrap();
			let config = config_from_js(cx, config)?;
			Ok(Environment::Dev(config))
		},
		_ => cx.throw_error("Invalid environment type"),
	}
}

/// Convert config from JSObject to Config
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `config_from_js` - Neon JsObject containing the config
/// # Returns
/// * `Config` - Config object
pub fn config_from_js(
	cx: &mut FunctionContext,
	config_from_js: Handle<JsObject>,
) -> NeonResult<Config> {
	let sdk_max_stale_friendship_days: Handle<JsNumber> =
		config_from_js.get(cx, "sdkMaxStaleFriendshipDays")?;
	let sdk_max_stale_friendship_days = sdk_max_stale_friendship_days.value(cx) as u32;

	let max_graph_page_size_bytes: Handle<JsNumber> =
		config_from_js.get(cx, "maxGraphPageSizeBytes")?;
	let max_graph_page_size_bytes = max_graph_page_size_bytes.value(cx) as u32;

	let max_page_id: Handle<JsNumber> = config_from_js.get(cx, "maxPageId")?;
	let max_page_id = max_page_id.value(cx) as u32;

	let max_key_page_size_bytes: Handle<JsNumber> =
		config_from_js.get(cx, "maxKeyPageSizeBytes")?;
	let max_key_page_size_bytes = max_key_page_size_bytes.value(cx) as u32;

	let schema_map: Handle<JsObject> = config_from_js.get(cx, "schemaMap")?;
	let schema_map = schema_map_from_js(cx, schema_map)?;

	let dsnp_versions: Handle<JsArray> = config_from_js.get(cx, "dsnpVersions")?;
	let dsnp_versions = dsnp_versions_from_js(cx, dsnp_versions)?;

	let graph_public_key_schema_id: Handle<JsNumber> =
		config_from_js.get(cx, "graphPublicKeySchemaId")?;
	let graph_public_key_schema_id = graph_public_key_schema_id.value(cx) as SchemaId;

	let config_from_js = Config {
		sdk_max_stale_friendship_days,
		max_graph_page_size_bytes,
		max_page_id,
		max_key_page_size_bytes,
		schema_map,
		graph_public_key_schema_id,
		dsnp_versions,
	};

	Ok(config_from_js)
}

/// Convert schema map from JSObject to HashMap
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `schema_map_from_js` - Neon JsObject containing the schema map
/// # Returns
/// * `HashMap<String, SchemaConfig>` - HashMap of schema configs
/// # Errors
/// * Throws a Neon error if the schema map cannot be converted
pub fn schema_map_from_js(
	cx: &mut FunctionContext,
	schema_map_from_js: Handle<JsObject>,
) -> NeonResult<std::collections::HashMap<SchemaId, SchemaConfig>> {
	let mut schema_map = std::collections::HashMap::new();

	let schema_map_keys = schema_map_from_js.get_own_property_names(cx)?;
	for key in schema_map_keys.to_vec(cx).unwrap() {
		let key_value: Handle<'_, JsString> = key.downcast_or_throw(cx)?;
		let key_str = key_value.value(cx);
		let key_u16 = match key_str.as_str().parse::<u16>() {
			Ok(key_u16) => key_u16,
			Err(_) => cx.throw_error("Invalid schema id")?,
		};
		let schema_config: Handle<'_, JsObject> = schema_map_from_js.get(cx, key)?;
		let dsnp_version_str: Handle<'_, JsString> = schema_config.get(cx, "dsnpVersion")?;
		let dsnp_version = match dsnp_version_str.value(cx).as_str() {
			"1.0" => DsnpVersion::Version1_0,
			_ => DsnpVersion::Version1_0,
		};
		let privacy_type: Handle<'_, JsString> = schema_config.get(cx, "privacyType")?;
		let privacy_type = match privacy_type.value(cx).as_str() {
			"public" => dsnp_graph_config::PrivacyType::Public,
			"private" => dsnp_graph_config::PrivacyType::Private,
			_ => cx.throw_error("Invalid privacy type")?,
		};

		let connection_type: Handle<'_, JsString> = schema_config.get(cx, "connectionType")?;
		let connection_type = match connection_type.value(cx).as_str() {
			"follow" => ConnectionType::Follow(privacy_type),
			"friendship" => ConnectionType::Friendship(privacy_type),
			_ => cx.throw_error("Invalid connection type")?,
		};

		let schema_config = SchemaConfig { dsnp_version, connection_type };
		schema_map.insert(key_u16, schema_config);
	}

	Ok(schema_map)
}

/// Convert dsnp versions from JSArray to Vec<DsnpVersion>
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `dsnp_versions_from_js` - Neon JsArray containing the dsnp versions
/// # Returns
/// * `Vec<DsnpVersion>` - Vec of DsnpVersion
/// # Errors
/// * Throws a Neon error if the dsnp versions cannot be converted
pub fn dsnp_versions_from_js(
	cx: &mut FunctionContext,
	dsnp_versions_from_js: Handle<JsArray>,
) -> NeonResult<Vec<DsnpVersion>> {
	let mut dsnp_versions = Vec::new();
	for i in 0..dsnp_versions_from_js.len(cx) {
		let dsnp_version_str: Handle<'_, JsString> = dsnp_versions_from_js.get(cx, i)?;
		let dsnp_version_str = dsnp_version_str.value(cx);
		let dsnp_version = match dsnp_version_str.as_str() {
			"1.0" => DsnpVersion::Version1_0,
			_ => cx.throw_error("Invalid dsnp version")?,
		};
		dsnp_versions.push(dsnp_version);
	}
	Ok(dsnp_versions)
}

/// Convert rust `Config` to JSObject
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `config` - Config object
/// # Returns
/// * `JsResult<JsObject>` - Neon JsObject containing the config
/// # Errors
pub fn config_to_js<'a, C: Context<'a>>(cx: &mut C, config: &Config) -> JsResult<'a, JsObject> {
	let obj = cx.empty_object();

	let sdk_max_stale_friendship_days = cx.number(config.sdk_max_stale_friendship_days);
	obj.set(cx, "sdkMaxStaleFriendshipDays", sdk_max_stale_friendship_days)?;

	let max_graph_page_size_bytes = cx.number(config.max_graph_page_size_bytes);
	obj.set(cx, "maxGraphPageSizeBytes", max_graph_page_size_bytes)?;

	let max_page_id = cx.number(config.max_page_id);
	obj.set(cx, "maxPageId", max_page_id)?;

	let max_key_page_size_bytes = cx.number(config.max_key_page_size_bytes);
	obj.set(cx, "maxKeyPageSizeBytes", max_key_page_size_bytes)?;

	let schema_map = cx.empty_object();
	for (schema_id, schema_config) in &config.schema_map {
		let schema_id_val = cx.number(*schema_id);
		let schema_config_obj = schema_config_to_js(cx, schema_config)?;
		schema_map.set(cx, schema_id_val, schema_config_obj)?;
	}
	obj.set(cx, "schemaMap", schema_map)?;

	let graph_public_key_schema_id = cx.number(config.graph_public_key_schema_id);
	obj.set(cx, "graphPublicKeySchemaId", graph_public_key_schema_id)?;

	let dsnp_versions = cx.empty_array();
	for (i, version) in config.dsnp_versions.iter().enumerate() {
		let version_val = cx.number(*version as u32);
		dsnp_versions.set(cx, i as u32, version_val)?;
	}
	obj.set(cx, "dsnpVersions", dsnp_versions)?;

	Ok(obj)
}

/// Convert rust `SchemaConfig` to JSObject
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `schema_config` - SchemaConfig object
/// # Returns
/// * `JsResult<JsObject>` - Neon JsObject containing the schema config
/// # Errors
/// * Throws a Neon error if the schema config cannot be converted
pub fn schema_config_to_js<'a, C: Context<'a>>(
	cx: &mut C,
	schema_config: &SchemaConfig,
) -> JsResult<'a, JsObject> {
	let obj = cx.empty_object();

	let dsnp_version = cx.number(schema_config.dsnp_version as u32);
	obj.set(cx, "dsnpVersion", dsnp_version)?;

	let connection_type_str = match schema_config.connection_type {
		ConnectionType::Follow(_) => cx.string("follow"),
		ConnectionType::Friendship(_) => cx.string("friendship"),
	};
	obj.set(cx, "connectionType", connection_type_str)?;

	let privacy_type_str = match schema_config.connection_type {
		ConnectionType::Follow(privacy) | ConnectionType::Friendship(privacy) => {
			let privacy_type_str = match privacy {
				dsnp_graph_config::PrivacyType::Public => cx.string("public"),
				dsnp_graph_config::PrivacyType::Private => cx.string("private"),
			};
			privacy_type_str
		},
	};
	obj.set(cx, "privacyType", privacy_type_str)?;
	Ok(obj)
}

/// Function to convert ImportBundle JsObject to ImportBundle struct
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `import_bundle_js` - Neon JsObject containing the import bundle
/// # Returns
/// * `JsResult<Vec<ImportBundle>>` - rust ImportBundle struct
/// # Errors
/// * Throws a Neon error if the import bundle cannot be converted
pub fn import_bundle_from_js<'a, C: Context<'a>>(
	cx: &mut C,
	import_bundle_js: Handle<'_, JsArray>,
) -> NeonResult<Vec<ImportBundle>> {
	let mut import_bundles: Vec<ImportBundle> = Vec::new();
	let import_bundle_js = import_bundle_js.to_vec(cx)?;
	for import_bundle in import_bundle_js {
		let import_bundle = import_bundle.downcast_or_throw::<JsObject, _>(cx)?;
		let import_bundle = import_bundle_from_js_object(cx, import_bundle)?;
		import_bundles.push(import_bundle);
	}
	Ok(import_bundles)
}

/// Function to convert ImportBundle JsObject to ImportBundle struct
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `import_bundle_js` - Neon JsObject containing the import bundle
/// # Returns
/// * `NeonResult<ImportBundle>` - rust ImportBundle struct
/// # Errors
/// * Throws a Neon error if the import bundle cannot be converted
pub fn import_bundle_from_js_object<'a, C: Context<'a>>(
	cx: &mut C,
	import_bundle_js: Handle<'_, JsObject>,
) -> NeonResult<ImportBundle> {
	let dsnp_user_id: Handle<'_, JsString> = import_bundle_js.get(cx, "dsnpUserId")?;
	let dsnp_user_id = match dsnp_user_id.value(cx).parse::<DsnpUserId>() {
		Ok(dsnp_user_id) => dsnp_user_id,
		Err(_) => cx.throw_error("Invalid dsnp user id")?,
	};
	let schema_id: Handle<'_, JsNumber> = import_bundle_js.get(cx, "schemaId")?;
	let schema_id = schema_id.value(cx) as SchemaId;
	let dsnp_keys: Option<Handle<'_, JsObject>> = import_bundle_js.get_opt(cx, "dsnpKeys")?;
	let dsnp_keys = match dsnp_keys {
		Some(keys) => Some(dsnp_keys_from_js(cx, keys)?),
		None => None,
	};

	let key_pairs: Option<Handle<'_, JsArray>> = import_bundle_js.get_opt(cx, "keyPairs")?;
	let key_pairs = match key_pairs {
		Some(kp) => key_pairs_from_js(cx, kp)?,
		None => Vec::new(),
	};

	let pages_res: Handle<'_, JsArray> = import_bundle_js.get(cx, "pages")?;
	let mut pages: Vec<PageData> = Vec::new();
	if pages_res.len(cx) > 0 {
		pages = pages_from_js(cx, pages_res)?;
	}

	let import_bundle = ImportBundle { dsnp_user_id, schema_id, dsnp_keys, key_pairs, pages };
	Ok(import_bundle)
}

/// Function to convert JsArray of PageData to Vec<PageData>
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `pages_js` - Neon JsArray of JsObjects
/// # Returns
/// * `Vec<PageData>` - Vec<PageData>
/// # Errors
/// * Throws a Neon error if the pages cannot be converted
pub fn pages_from_js<'a, C: Context<'a>>(
	cx: &mut C,
	pages_js: Handle<'_, JsArray>,
) -> NeonResult<Vec<PageData>> {
	let mut pages: Vec<PageData> = Vec::new();
	let pages_js = pages_js.to_vec(cx)?;
	for page in pages_js {
		let page = page.downcast_or_throw::<JsObject, _>(cx)?;
		let page = page_from_js(cx, page)?;
		pages.push(page);
	}
	Ok(pages)
}

/// Function to convert JsObject of PageData to PageData
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `page_js` - Neon JsObject
/// # Returns
/// * `PageData` - PageData
/// # Errors
/// * Throws a Neon error if the page cannot be converted
pub fn page_from_js<'a, C: Context<'a>>(
	cx: &mut C,
	page_js: Handle<'_, JsObject>,
) -> NeonResult<PageData> {
	let page_id: Handle<'_, JsNumber> = page_js.get(cx, "pageId")?;
	let page_id = page_id.value(cx) as PageId;

	let content_hash: Handle<'_, JsNumber> = page_js.get(cx, "contentHash")?;
	let content_hash = content_hash.value(cx) as PageHash;

	let content: Handle<'_, JsTypedArray<u8>> = page_js.get(cx, "content")?;
	let content = content.as_slice(cx).to_vec();
	Ok(PageData { page_id, content_hash, content })
}

/// Function to convert JsArray of GraphKeyPair to Vec<GraphKeyPair>
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `key_pairs_js` - Neon JsArray of JsObjects
/// # Returns
/// * `Vec<GraphKeyPair>` - Vec<GraphKeyPair>
/// # Errors
/// * Throws a Neon error if the key pairs cannot be converted
pub fn key_pairs_from_js<'a, C: Context<'a>>(
	cx: &mut C,
	key_pairs_js: Handle<'_, JsArray>,
) -> NeonResult<Vec<GraphKeyPair>> {
	let mut key_pairs: Vec<GraphKeyPair> = Vec::new();
	let key_pairs_js = key_pairs_js.to_vec(cx)?;
	for key_pair in key_pairs_js {
		let key_pair = key_pair.downcast_or_throw::<JsObject, _>(cx)?;
		let key_pair = key_pair_from_js(cx, key_pair)?;
		key_pairs.push(key_pair);
	}
	Ok(key_pairs)
}

/// Function to convert JsObject of GraphKeyPair to GraphKeyPair
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `key_pair_js` - Neon JsObject
/// # Returns
/// * `GraphKeyPair` - GraphKeyPair
/// # Errors
/// * Throws a Neon error if the key pair cannot be converted
pub fn key_pair_from_js<'a, C: Context<'a>>(
	cx: &mut C,
	key_pair_js: Handle<'_, JsObject>,
) -> NeonResult<GraphKeyPair> {
	let key_type: Handle<'_, JsNumber> = key_pair_js.get(cx, "keyType")?;
	let key_type = key_type.value(cx);
	let key_type = match key_type as u8 {
		0 => dsnp_graph_config::GraphKeyType::X25519,
		_ => cx.throw_error("Invalid key type")?,
	};

	let public_key: Handle<'_, JsTypedArray<u8>> = key_pair_js.get(cx, "publicKey")?;
	let public_key = public_key.as_slice(cx).to_vec();

	let secret_key: Handle<'_, JsTypedArray<u8>> = key_pair_js.get(cx, "secretKey")?;
	let secret_key = secret_key.as_slice(cx).to_vec();

	Ok(GraphKeyPair { key_type, public_key, secret_key })
}

/// Function to convert GraphKeyPair to JsObject
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `key_pair` - GraphKeyPair
/// # Returns
/// * `JsResult<JsObject>` - Neon JsObject
/// # Errors
/// * Throws a Neon error if the key pair cannot be converted
pub fn keypair_to_js<'a, C: Context<'a>>(
	cx: &mut C,
	key_pair: &GraphKeyPair,
) -> NeonResult<Handle<'a, JsObject>> {
	let obj = cx.empty_object();
	let key_type = match key_pair.key_type {
		dsnp_graph_config::GraphKeyType::X25519 => cx.number(0),
	};
	obj.set(cx, "keyType", key_type)?;

	let mut public_key = cx.buffer(key_pair.public_key.len() as usize)?;
	public_key.as_mut_slice(cx).copy_from_slice(&key_pair.public_key);
	obj.set(cx, "publicKey", public_key)?;

	let mut secret_key = cx.buffer(key_pair.secret_key.len() as usize)?;
	secret_key.as_mut_slice(cx).copy_from_slice(&key_pair.secret_key);
	obj.set(cx, "secretKey", secret_key)?;

	Ok(obj)
}

/// Function to convert DsnpKeys JsObject to DsnpKeys struct
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `dsnp_keys_js` - Neon JsObject containing the dsnp keys
/// # Returns
/// * `NeonResult<DsnpKeys>` - rust DsnpKeys struct
/// # Errors
/// * Throws a Neon error if the dsnp keys cannot be converted
pub fn dsnp_keys_from_js<'a, C: Context<'a>>(
	cx: &mut C,
	dsnp_keys_js: Handle<'_, JsObject>,
) -> NeonResult<DsnpKeys> {
	let dsnp_user_id: Handle<'_, JsString> = dsnp_keys_js.get(cx, "dsnpUserId")?;
	let dsnp_user_id = match dsnp_user_id.value(cx).parse::<DsnpUserId>() {
		Ok(dsnp_user_id) => dsnp_user_id,
		Err(_) => cx.throw_error("Invalid dsnp user id")?,
	};

	let keys_hash: Handle<'_, JsNumber> = dsnp_keys_js.get(cx, "keysHash")?;
	let keys_hash = keys_hash.value(cx) as PageHash;

	let keys: Handle<'_, JsArray> = dsnp_keys_js.get(cx, "keys")?;
	let keys: Vec<KeyData> = keys_from_js(cx, keys)?;

	Ok(DsnpKeys { dsnp_user_id, keys_hash, keys })
}
/// Function to convert JsArray of KeyData to Vec<KeyData>
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `keys_js` - Neon JsArray of JsObjects
/// # Returns
/// * `Vec<KeyData>` - Vec<KeyData>
/// # Errors
/// * Throws a Neon error if the keys cannot be converted
pub fn keys_from_js<'a, C: Context<'a>>(
	cx: &mut C,
	keys_js: Handle<'_, JsArray>,
) -> NeonResult<Vec<KeyData>> {
	let mut keys: Vec<KeyData> = Vec::new();
	let keys_js = keys_js.to_vec(cx)?;
	for key in keys_js {
		let key = key.downcast_or_throw::<JsObject, _>(cx)?;
		let index: Handle<'_, JsNumber> = key.get(cx, "index")?;
		let index = index.value(cx) as u16;
		let content: Handle<'_, JsTypedArray<u8>> = key.get(cx, "content")?;
		let content = content.as_slice(cx).to_vec();
		keys.push(KeyData { index, content });
	}
	Ok(keys)
}

/// Function to convert rust Vec<Update> to JsArray of JsObjects
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `updates` - rust Vec<Update>
/// # Returns
/// * `JsResult<JsArray>` - Neon JsArray of JsObjects
/// # Errors
/// * Throws a Neon error if the updates cannot be converted
pub fn updates_to_js<'a, C: Context<'a>>(
	cx: &mut C,
	updates: Vec<Update>,
) -> JsResult<'a, JsArray> {
	let updates_js = cx.empty_array();
	for (i, update) in updates.iter().enumerate() {
		let update_js = update_to_js(cx, update)?;
		updates_js.set(cx, i as u32, update_js)?;
	}
	Ok(updates_js)
}

/// Function to convert rust Update to JsObject
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `update` - rust Update
/// # Returns
/// * `JsResult<JsObject>` - Neon JsObject
/// # Errors
/// * Throws a Neon error if the update cannot be converted
pub fn update_to_js<'a, C: Context<'a>>(
	cx: &mut C,
	update: &Update,
) -> NeonResult<Handle<'a, JsObject>> {
	let obj = cx.empty_object();
	match update {
		Update::AddKey { owner_dsnp_user_id, prev_hash, payload } => {
			let type_update = cx.string("AddKey");
			obj.set(cx, "type", type_update)?;
			let owner_dsnp_user_id = cx.string(owner_dsnp_user_id.to_string());
			obj.set(cx, "ownerDsnpUserId", owner_dsnp_user_id)?;

			let prev_hash = cx.number(*prev_hash);
			obj.set(cx, "prevHash", prev_hash)?;
			let len = payload.len().try_into().unwrap();
			let mut payload_buffer = cx.buffer(len)?;

			payload_buffer.as_mut_slice(cx).copy_from_slice(&payload);
			obj.set(cx, "payload", payload_buffer)?;
		},
		Update::PersistPage { owner_dsnp_user_id, schema_id, page_id, prev_hash, payload } => {
			let type_update = cx.string("PersistPage");
			obj.set(cx, "type", type_update)?;
			let owner_dsnp_user_id = cx.string(owner_dsnp_user_id.to_string());
			obj.set(cx, "ownerDsnpUserId", owner_dsnp_user_id)?;

			let schema_id = cx.number(*schema_id);
			obj.set(cx, "schemaId", schema_id)?;

			let page_id = cx.number(*page_id);
			obj.set(cx, "pageId", page_id)?;

			let prev_hash = cx.number(*prev_hash);
			obj.set(cx, "prevHash", prev_hash)?;

			let len = payload.len().try_into().unwrap();
			let mut payload_buffer = cx.buffer(len)?;

			payload_buffer.as_mut_slice(cx).copy_from_slice(&payload);
			obj.set(cx, "payload", payload_buffer)?;
		},
		Update::DeletePage { owner_dsnp_user_id, schema_id, page_id, prev_hash } => {
			let type_update = cx.string("DeletePage");
			obj.set(cx, "type", type_update)?;
			let owner_dsnp_user_id = cx.string(owner_dsnp_user_id.to_string());
			obj.set(cx, "ownerDsnpUserId", owner_dsnp_user_id)?;

			let schema_id = cx.number(*schema_id);
			obj.set(cx, "schemaId", schema_id)?;

			let page_id = cx.number(*page_id);
			obj.set(cx, "pageId", page_id)?;

			let prev_hash = cx.number(*prev_hash);
			obj.set(cx, "prevHash", prev_hash)?;
		},
	};
	Ok(obj)
}

/// Function to convert Vec<DsnpGraphEdge> to JsArray of JsObjects
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `edges` - Vec<DsnpGraphEdge>
/// # Returns
/// * `JsResult<JsArray>` - Neon JsArray of JsObjects
/// # Errors
/// * Throws a Neon error if the edges cannot be converted
pub fn connections_to_js<'a, C: Context<'a>>(
	cx: &mut C,
	edges: Vec<DsnpGraphEdge>,
) -> JsResult<'a, JsArray> {
	let edges_js = cx.empty_array();
	for (i, edge) in edges.iter().enumerate() {
		let edge_js = connection_to_js(cx, edge)?;
		edges_js.set(cx, i as u32, edge_js)?;
	}
	Ok(edges_js)
}

/// Function to convert DsnpGraphEdge to JsObject
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `edge` - DsnpGraphEdge
/// # Returns
/// * `JsResult<JsObject>` - Neon JsObject
/// # Errors
/// * Throws a Neon error if the edge cannot be converted
pub fn connection_to_js<'a, C: Context<'a>>(
	cx: &mut C,
	edge: &DsnpGraphEdge,
) -> JsResult<'a, JsObject> {
	let obj = cx.empty_object();
	let dsnp_user_id = cx.string(edge.user_id.to_string());
	obj.set(cx, "userId", dsnp_user_id)?;
	let since: Handle<'_, JsNumber> = cx.number(edge.since as f64);
	obj.set(cx, "since", since)?;
	Ok(obj)
}

/// Function to convert JSArray of Action to Vec<Action>
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `actions_js` - Neon JsArray of JsObjects
/// # Returns
/// * `Vec<Action>` - Vec<Action>
/// # Errors
/// * Throws a Neon error if the actions cannot be converted
pub fn actions_from_js<'a, C: Context<'a>>(
	cx: &mut C,
	actions_js: Handle<'_, JsArray>,
) -> NeonResult<Vec<Action>> {
	let mut actions: Vec<Action> = Vec::new();
	let actions_vec = actions_js.to_vec(cx)?;
	for action in actions_vec {
		let action = action.downcast_or_throw::<JsObject, _>(cx)?;
		let action = action_from_js(cx, action)?;
		actions.push(action);
	}
	Ok(actions)
}

/// Function to convert JsObject of Action to Action
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `action_js` - Neon JsObject
/// # Returns
/// * `Action` - Action
/// # Errors
/// * Throws a Neon error if the action cannot be converted
pub fn action_from_js<'a, C: Context<'a>>(
	cx: &mut C,
	action_js: Handle<'_, JsObject>,
) -> NeonResult<Action> {
	let action_type: Handle<'_, JsString> = action_js.get(cx, "type")?;
	let action_type = action_type.value(cx);
	let action = match action_type.as_str() {
		"Connect" => {
			let owner_dsnp_user_id: Handle<'_, JsString> = action_js.get(cx, "ownerDsnpUserId")?;
			let owner_dsnp_user_id = match owner_dsnp_user_id.value(cx).parse::<DsnpUserId>() {
				Ok(owner_dsnp_user_id) => owner_dsnp_user_id,
				Err(_) => cx.throw_error("Invalid dsnp user id")?,
			};

			let dsnp_keys: Option<DsnpKeys> = match action_js.get_opt(cx, "dsnpKeys") {
				Ok(dsnp_keys) => match dsnp_keys {
					Some(dsnp_keys) => {
						let dsnp_keys: DsnpKeys = dsnp_keys_from_js(cx, dsnp_keys)?;
						Some(dsnp_keys)
					},
					None => None,
				},
				Err(_) => None,
			};
			let connection: Handle<'_, JsObject> = action_js.get(cx, "connection")?;
			let connection: Connection = connection_from_js(cx, connection)?;

			Action::Connect { owner_dsnp_user_id, dsnp_keys, connection }
		},
		"Disconnect" => {
			let owner_dsnp_user_id: Handle<'_, JsString> = action_js.get(cx, "ownerDsnpUserId")?;
			let owner_dsnp_user_id = match owner_dsnp_user_id.value(cx).parse::<DsnpUserId>() {
				Ok(owner_dsnp_user_id) => owner_dsnp_user_id,
				Err(_) => cx.throw_error("Invalid dsnp user id")?,
			};

			let connection: Handle<'_, JsObject> = action_js.get(cx, "connection")?;
			let connection: Connection = connection_from_js(cx, connection)?;

			Action::Disconnect { owner_dsnp_user_id, connection }
		},
		"AddGraphKey" => {
			let owner_dsnp_user_id: Handle<'_, JsString> = action_js.get(cx, "ownerDsnpUserId")?;
			let owner_dsnp_user_id = match owner_dsnp_user_id.value(cx).parse::<DsnpUserId>() {
				Ok(owner_dsnp_user_id) => owner_dsnp_user_id,
				Err(_) => cx.throw_error("Invalid dsnp user id")?,
			};

			let new_public_key: Handle<'_, JsTypedArray<u8>> = action_js.get(cx, "newPublicKey")?;
			let new_public_key = new_public_key.as_slice(cx).to_vec();

			Action::AddGraphKey { owner_dsnp_user_id, new_public_key }
		},
		_ => cx.throw_error("Invalid action type")?,
	};
	Ok(action)
}

/// Function to convert JsObject of Connection to Connection
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `connection_js` - Neon JsObject
/// # Returns
/// * `Connection` - Connection
/// # Errors
/// * Throws a Neon error if the connection cannot be converted
pub fn connection_from_js<'a, C: Context<'a>>(
	cx: &mut C,
	connection_js: Handle<'_, JsObject>,
) -> NeonResult<Connection> {
	let dsnp_user_id: Handle<'_, JsString> = connection_js.get(cx, "dsnpUserId")?;
	let dsnp_user_id = match dsnp_user_id.value(cx).parse::<DsnpUserId>() {
		Ok(dsnp_user_id) => dsnp_user_id,
		Err(_) => cx.throw_error("Invalid dsnp user id")?,
	};

	let schema_id: Handle<'_, JsNumber> = connection_js.get(cx, "schemaId")?;
	let schema_id = schema_id.value(cx) as SchemaId;
	Ok(Connection { dsnp_user_id, schema_id })
}

/// Function to convert Vec<DsnpPublicKey> to JsArray of JsObjects
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `public_keys` - Vec<DsnpPublicKey>
/// # Returns
/// * `JsResult<JsArray>` - Neon JsArray of JsObjects
/// # Errors
/// * Throws a Neon error if the public keys cannot be converted
pub fn public_keys_to_js<'a, C: Context<'a>>(
	cx: &mut C,
	public_keys: Vec<DsnpPublicKey>,
) -> JsResult<'a, JsArray> {
	let public_keys_js = cx.empty_array();
	for (i, public_key) in public_keys.iter().enumerate() {
		let public_key_js = public_key_to_js(cx, public_key)?;
		public_keys_js.set(cx, i as u32, public_key_js)?;
	}
	Ok(public_keys_js)
}

/// Function to convert DsnpPublicKey to JsObject
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `public_key` - DsnpPublicKey
/// # Returns
/// * `JsResult<JsObject>` - Neon JsObject
/// # Errors
/// * Throws a Neon error if the public key cannot be converted
pub fn public_key_to_js<'a, C: Context<'a>>(
	cx: &mut C,
	public_key: &DsnpPublicKey,
) -> NeonResult<Handle<'a, JsObject>> {
	let obj = cx.empty_object();

	let len = public_key.key.len().try_into().unwrap();
	let mut key_buffer = cx.buffer(len)?;
	key_buffer.as_mut_slice(cx).copy_from_slice(&public_key.key);
	obj.set(cx, "key", key_buffer)?;

	let public_key_id = public_key.key_id.unwrap_or_default();
	let key_id = cx.string(public_key_id.to_string());
	obj.set(cx, "keyId", key_id)?;

	Ok(obj)
}
