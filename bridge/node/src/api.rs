use dsnp_graph_config::{errors::DsnpGraphError, Config as RustConfig, SchemaId};
use dsnp_graph_core::{
	api::api::{GraphAPI, GraphState},
	dsnp::dsnp_types::DsnpUserId,
};
use neon::prelude::*;
use std::{panic, sync::Mutex};

// Collection of GraphStates
#[allow(clippy::vec_box)]
static GRAPH_STATES: Mutex<Vec<Box<GraphState>>> = Mutex::new(Vec::new());

/// Neon implementation of print_hello_graph function
fn print_hello_graph(mut cx: FunctionContext) -> JsResult<JsUndefined> {
	println!("Hello, Graph!");
	Ok(cx.undefined())
}

/// Get graph config from the environment
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `env` - Neon Environment object
/// # Returns
/// * `JsResult<JsObject>` - Neon JsObject containing the graph config
pub fn get_graph_config(mut cx: FunctionContext) -> JsResult<JsObject> {
	let environment_from_js = cx.argument::<JsObject>(0)?;
	let config_object = cx.empty_object();
	Ok(config_object)
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
pub unsafe fn create_graph_state(mut cx: FunctionContext) -> JsResult<JsObject> {
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
pub unsafe fn create_graph_state_with_capacity(mut cx: FunctionContext) -> JsResult<JsObject> {
	return cx.throw_error("Not implemented")
}
