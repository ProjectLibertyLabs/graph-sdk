use dsnp_graph_config::{Config, ConnectionType, Environment, SchemaConfig};
use dsnp_graph_core::api::api_types::ImportBundle;
use neon::{
	handle::Handle,
	object::Object,
	prelude::{Context, FunctionContext},
	result::JsResult,
	types::{JsArray, JsObject, JsString, Value},
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
