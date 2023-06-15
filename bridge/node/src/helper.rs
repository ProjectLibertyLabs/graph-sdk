use dsnp_graph_config::{Config, ConnectionType, Environment, SchemaConfig};
use dsnp_graph_core::{
	api::api_types::{Action, Connection, DsnpKeys, ImportBundle, Update},
	dsnp::dsnp_types::{DsnpGraphEdge, DsnpPublicKey},
};
use neon::{
	handle::Handle,
	object::Object,
	prelude::{Context, FunctionContext},
	result::JsResult,
	types::{JsArray, JsNumber, JsObject, JsString, Value},
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
) -> Environment {
	let environment_type_str: Handle<JsString> =
		environment_from_js.get(cx, "environmentType").unwrap_or(cx.string(""));

	match environment_type_str.value(cx).as_str() {
		"mainnet" => Environment::Mainnet,
		"rococo" => Environment::Rococo,
		"dev" => {
			let config: Handle<JsObject> =
				environment_from_js.get(cx, "config").unwrap_or(cx.empty_object());
			let config = config_from_js(cx, config);
			Environment::Dev(config)
		},
		_ => Environment::Rococo,
	}
}

/// Convert config from JSObject to Config
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `config_from_js` - Neon JsObject containing the config
/// # Returns
/// * `Config` - Config object
pub fn config_from_js(cx: &mut FunctionContext, config_from_js: Handle<JsObject>) -> Config {
	let config_str: Handle<JsString> = config_from_js.to_string(cx).unwrap_or(cx.string(""));
	let config_str = config_str.value(cx);
	Config::try_from(config_str.as_str()).unwrap()
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

	let sdk_max_users_graph_size = cx.number(config.sdk_max_users_graph_size);
	obj.set(cx, "sdkMaxUsersGraphSize", sdk_max_users_graph_size)?;

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
pub fn import_bundle_from_js(
	cx: &mut FunctionContext,
	import_bundle_js: Handle<JsArray>,
) -> Vec<ImportBundle> {
	let mut import_bundles: Vec<ImportBundle> = Vec::new();
	for i in 0..import_bundle_js.len(cx) {
		let import_bundle = import_bundle_js.get(cx, i).unwrap_or(cx.empty_object());
		let import_bundle_str: Handle<JsString> =
			import_bundle.to_string(cx).unwrap_or(cx.string(""));
		let import_bundle_str = import_bundle_str.value(cx);
		import_bundles.push(ImportBundle::try_from(import_bundle_str.as_str()).unwrap());
	}
	import_bundles
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
pub fn update_to_js<'a, C: Context<'a>>(cx: &mut C, update: &Update) -> JsResult<'a, JsObject> {
	let obj = cx.empty_object();
	match update {
		Update::AddKey { owner_dsnp_user_id, prev_hash, payload } => {
			let type_update = cx.string("AddKey");
			obj.set(cx, "type", type_update)?;
			let owner_dsnp_user_id = cx.number(*owner_dsnp_user_id as f64);
			obj.set(cx, "ownerDsnpUserId", owner_dsnp_user_id)?;

			let prev_hash = cx.number(*prev_hash);
			obj.set(cx, "prevHash", prev_hash)?;
			let payload_array: Handle<JsArray> = vec_u8_to_js_array(cx, payload);
			obj.set(cx, "payload", payload_array)?;
		},
		Update::PersistPage { owner_dsnp_user_id, schema_id, page_id, prev_hash, payload } => {
			let type_update = cx.string("PersistPage");
			obj.set(cx, "type", type_update)?;
			let owner_dsnp_user_id = cx.number(*owner_dsnp_user_id as f64);
			obj.set(cx, "ownerDsnpUserId", owner_dsnp_user_id)?;

			let schema_id = cx.number(*schema_id);
			obj.set(cx, "schemaId", schema_id)?;

			let page_id = cx.number(*page_id);
			obj.set(cx, "pageId", page_id)?;

			let prev_hash = cx.number(*prev_hash);
			obj.set(cx, "prevHash", prev_hash)?;

			let payload_array: Handle<JsArray> = vec_u8_to_js_array(cx, payload);
			obj.set(cx, "payload", payload_array)?;
		},
		Update::DeletePage { owner_dsnp_user_id, schema_id, page_id, prev_hash } => {
			let type_update = cx.string("DeletePage");
			obj.set(cx, "type", type_update)?;
			let owner_dsnp_user_id = cx.number(*owner_dsnp_user_id as f64);
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
	let dsnp_user_id = cx.number(edge.user_id as f64);
	obj.set(cx, "userId", dsnp_user_id)?;
	let since = cx.number(edge.since as f64);
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
pub fn actions_from_js(cx: &mut FunctionContext, actions_js: Handle<JsArray>) -> Vec<Action> {
	let mut actions: Vec<Action> = Vec::new();
	for i in 0..actions_js.len(cx) {
		let action = actions_js.get(cx, i).unwrap_or(cx.empty_object());
		let action_type = action.get(cx, "type").unwrap_or(cx.string(""));
		let action_type = action_type.value(cx);

		match action_type.as_str() {
			"Connect" => {
				let action_type = Action::Connect {
					owner_dsnp_user_id: action
						.get(cx, "ownerDsnpUserId")
						.unwrap_or(cx.number(0.0))
						.value(cx) as u64,
					dsnp_keys: Some(
						DsnpKeys::try_from(
							action
								.get(cx, "dsnpKeys")
								.unwrap_or(cx.empty_object())
								.to_string(cx)
								.unwrap_or(cx.string(""))
								.value(cx)
								.as_str(),
						)
						.unwrap(),
					),
					connection: Connection::try_from(
						action
							.get(cx, "connection")
							.unwrap_or(cx.empty_object())
							.to_string(cx)
							.unwrap_or(cx.string(""))
							.value(cx)
							.as_str(),
					)
					.unwrap(),
				};
				actions.push(action_type);
			},
			"Disconnect" => {
				let action_type = Action::Disconnect {
					owner_dsnp_user_id: action
						.get(cx, "ownerDsnpUserId")
						.unwrap_or(cx.number(0.0))
						.value(cx) as u64,
					connection: Connection::try_from(
						action
							.get(cx, "connection")
							.unwrap_or(cx.empty_object())
							.to_string(cx)
							.unwrap_or(cx.string(""))
							.value(cx)
							.as_str(),
					)
					.unwrap(),
				};
				actions.push(action_type);
			},
			"AddGraphKey" => {
				let public_key = action.get(cx, "newPublicKey").unwrap_or(cx.empty_array());
				let action_type = Action::AddGraphKey {
					owner_dsnp_user_id: action
						.get(cx, "ownerDsnpUserId")
						.unwrap_or(cx.number(0.0))
						.value(cx) as u64,
					// JsArray<u8> to Vec<u8>
					new_public_key: vec_u8_from_js(cx, public_key),
				};
				actions.push(action_type);
			},

			_ => (),
		}
	}
	actions
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
) -> JsResult<'a, JsObject> {
	let obj = cx.empty_object();
	let key: Handle<JsArray> = vec_u8_to_js_array(cx, &public_key.key);
	obj.set(cx, "key", key)?;
	let key_id = cx.number(public_key.key_id.unwrap_or(0) as f64);
	obj.set(cx, "keyId", key_id)?;
	Ok(obj)
}

/// Function to convert J
/// Function to convert Vec<u8> to JsArray
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `data` - Vec<u8> to convert
/// # Returns
/// * `Handle<JsArray>` - JsArray containing the converted data
fn vec_u8_to_js_array<'a, C: Context<'a>>(cx: &mut C, data: &[u8]) -> Handle<'a, JsArray> {
	let js_array = JsArray::new(cx, data.len() as u32);

	for (index, value) in data.iter().enumerate() {
		let js_number = JsNumber::new(cx, *value as f64);
		js_array.set(cx, index as u32, js_number).unwrap();
	}

	js_array
}

/// Function to convert JSArray to Vec<u8>
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `data` - JsArray to convert
/// # Returns
/// * `Vec<u8>` - Vec<u8> containing the converted data
/// # Errors
/// * Throws a Neon error if the data cannot be converted
pub fn vec_u8_from_js<'a, C: Context<'a>>(cx: &mut C, data: Handle<'a, JsArray>) -> Vec<u8> {
	let mut vec: Vec<u8> = Vec::new();
	for i in 0..data.len(cx) {
		let value = data.get(cx, i).unwrap_or(cx.number(0.0));
		let value = value.value(cx) as u8;
		vec.push(value);
	}
	vec
}
