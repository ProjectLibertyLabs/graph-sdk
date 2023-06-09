use crate::helper::*;
use dsnp_graph_config::Config;
use neon::prelude::*;

// Collection of GraphStates
//#[allow(clippy::vec_box)]
//static GRAPH_STATES: Mutex<Vec<Box<GraphState>>> = Mutex::new(Vec::new());

/// Neon implementation of print_hello_graph function
pub fn print_hello_graph(mut cx: FunctionContext) -> JsResult<JsString> {
	println!("Hello, Graph!");
	Ok(cx.string("Hello, Graph!"))
}

/// Get graph config from the environment
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `env` - Neon Environment object
/// # Returns
/// * `JsResult<JsObject>` - Neon JsObject containing the graph config
pub fn get_graph_config(mut cx: FunctionContext) -> JsResult<JsObject> {
	let environment_obj = cx.argument::<JsObject>(0)?;

	let environment = unsafe { environment_from_js(&mut cx, environment_obj) };
	let config: &Config = environment.get_config();

	// Convert Config to JsObject
	let config_js = config_to_js(&mut cx, config)?;

	Ok(config_js)
}

/// Create a new graph state
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `env` - Neon Environment object
/// # Returns
/// * `JsResult<JsObject>` - Neon JsObject containing the graph state
/// # Errors
/// * Throws a Neon error if the graph state cannot be created
/// # Safety
pub fn create_graph_state(mut cx: FunctionContext) -> JsResult<JsObject> {
	return cx.throw_error("Not implemented")
}

/// Create graph state with capacity
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `env` - Neon Environment object
/// # Returns
/// * `JsResult<JsObject>` - Neon JsObject containing the graph state
/// # Errors
/// * Throws a Neon error if the graph state cannot be created
pub fn create_graph_state_with_capacity(mut cx: FunctionContext) -> JsResult<JsObject> {
	return cx.throw_error("Not implemented")
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
	cx.export_function("printHelloGraph", print_hello_graph)?;
	cx.export_function("getGraphConfig", get_graph_config)?;
	cx.export_function("createGraphState", create_graph_state)?;
	cx.export_function("createGraphStateWithCapacity", create_graph_state_with_capacity)?;
	Ok(())
}
