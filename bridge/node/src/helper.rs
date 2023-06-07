use dsnp_graph_config::{Config, Environment};
use neon::{
	handle::Handle,
	object::Object,
	prelude::{Context, FunctionContext},
	types::{JsObject, JsString, Value},
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
