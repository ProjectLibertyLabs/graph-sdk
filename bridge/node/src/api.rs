use crate::helper::*;
use dsnp_graph_config::Config;
use dsnp_graph_core::api::api::GraphState;
use neon::prelude::*;
use once_cell::sync::Lazy;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
};

// Collection of GraphStates
#[allow(clippy::vec_box)]
static GRAPH_STATES: Lazy<Mutex<HashMap<usize, Arc<Mutex<GraphState>>>>> =
	Lazy::new(|| Mutex::new(HashMap::new()));

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
/// * `env` - Neon Environment object extracted from context
/// # Returns
/// * `JsResult<JsObject>` - Neon JsObject containing the graph state
/// # Errors
/// * Throws a Neon error if the graph state cannot be created
/// # Safety
pub fn initialize_graph_state(mut cx: FunctionContext) -> JsResult<JsNumber> {
	let environment_obj = cx.argument::<JsObject>(0)?;
	let rust_environment = unsafe { environment_from_js(&mut cx, environment_obj) };
	let graph_state = GraphState::new(rust_environment);

	// Generate a unique identifier for the graph state
	let graph_state_id = {
		let mut states = GRAPH_STATES.lock().unwrap();
		let next_id = states.len();
		states.insert(next_id, Arc::new(Mutex::new(graph_state)));
		next_id
	};

	Ok(cx.number(graph_state_id as f64))
}

/// Create graph state with capacity
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `env` - Neon Environment object extracted from context
/// # Returns
/// * `JsResult<JsObject>` - Neon JsObject containing the graph state
/// # Errors
/// * Throws a Neon error if the graph state cannot be created
pub fn initialize_graph_state_with_capacity(mut cx: FunctionContext) -> JsResult<JsNumber> {
	let environment_obj = cx.argument::<JsObject>(0)?;
	let capacity = cx.argument::<JsNumber>(1)?;
	let capacity = capacity.value(&mut cx) as usize;
	let rust_environment = unsafe { environment_from_js(&mut cx, environment_obj) };
	let graph_state = GraphState::with_capacity(rust_environment, capacity);

	// Generate a unique identifier for the graph state
	let graph_state_id = {
		let mut states = GRAPH_STATES.lock().unwrap();
		let next_id = states.len();
		states.insert(next_id, Arc::new(Mutex::new(graph_state)));
		next_id
	};

	Ok(cx.number(graph_state_id as f64))
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
	cx.export_function("printHelloGraph", print_hello_graph)?;
	cx.export_function("getGraphConfig", get_graph_config)?;
	cx.export_function("initializeGraphState", initialize_graph_state)?;
	cx.export_function("initializeGraphStateWithCapacity", initialize_graph_state_with_capacity)?;
	Ok(())
}
