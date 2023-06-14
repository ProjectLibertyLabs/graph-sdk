use crate::helper::*;
use dsnp_graph_config::{Config, DsnpUserId};
use dsnp_graph_core::api::api::{GraphState, GraphAPI};
use neon::prelude::*;
use once_cell::sync::Lazy;
use std::{
	collections::HashMap,
	sync::{Arc, Mutex},
};

/// Global counter for graph state ids
static NEXT_GRAPH_STATE_ID: Lazy<Mutex<usize>> = Lazy::new(|| Mutex::new(0));

/// Collection of GraphStates
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
		let mut next_id = NEXT_GRAPH_STATE_ID.lock().unwrap();
		let id = *next_id;
		*next_id = next_id.wrapping_add(1);
		id
	};
	{
		let mut states = GRAPH_STATES.lock().unwrap();
		states.insert(graph_state_id, Arc::new(Mutex::new(graph_state)));
	}

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
		let mut next_id = NEXT_GRAPH_STATE_ID.lock().unwrap();
		let id = *next_id;
		*next_id = next_id.wrapping_add(1);
		id
	};
	{
		let mut states = GRAPH_STATES.lock().unwrap();
		states.insert(graph_state_id, Arc::new(Mutex::new(graph_state)));
	}

	Ok(cx.number(graph_state_id as f64))
}

/// Get the capacity of the graph state
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `graph_state_id` - Unique identifier for the graph state
/// # Returns
/// * `JsResult<JsNumber>` - Neon JsNumber containing the capacity of the graph state
/// # Errors
/// * Throws a Neon error if the graph state cannot be found
pub fn get_graph_capacity(mut cx: FunctionContext) -> JsResult<JsNumber> {
	let graph_state_id = cx.argument::<JsNumber>(0)?;
	let graph_state_id = graph_state_id.value(&mut cx) as usize;

	let states = GRAPH_STATES.lock().unwrap();
	let graph_state = states.get(&graph_state_id);
	if graph_state.is_none() {
		return cx.throw_error("Graph state not found")
	}
	let graph_state = graph_state.unwrap();
	let graph_state = graph_state.lock().unwrap();
	let capacity = graph_state.capacity();

	Ok(cx.number(capacity as f64))
}
/// Get graph users count for given graph state
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `graph_state_id` - Unique identifier for the graph state
/// # Returns
/// * `JsResult<JsNumber>` - Neon JsNumber containing the graph users count
/// # Errors
/// * Throws a Neon error if the graph state cannot be found
pub fn get_graph_users_count(mut cx: FunctionContext) -> JsResult<JsNumber> {
	let graph_state_id = cx.argument::<JsNumber>(0)?;
	let graph_state_id = graph_state_id.value(&mut cx) as usize;

	let states = GRAPH_STATES.lock().unwrap();
	let graph_state = states.get(&graph_state_id);
	if graph_state.is_none() {
		return cx.throw_error("Graph state not found")
	}
	let graph_state = graph_state.unwrap();
	let graph_state = graph_state.lock().unwrap();
	let users_count = graph_state.len();

	Ok(cx.number(users_count as f64))
}

/// Check if graph contains user
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `graph_state_id` - Unique identifier for the graph state
/// * `dsnpt_user_id` - DSNP user id
/// # Returns
/// * `JsResult<JsBoolean>` - Neon JsBoolean
/// # Errors
/// * Throws a Neon error if the graph state cannot be found
pub fn contains_user_graph(mut cx: FunctionContext) -> JsResult<JsBoolean> {
	let graph_state_id = cx.argument::<JsNumber>(0)?;
	let graph_state_id = graph_state_id.value(&mut cx) as usize;
	let dsnp_user_id = cx.argument::<JsNumber>(1)?;
	let dsnp_user_id = dsnp_user_id.value(&mut cx) as DsnpUserId;

	let states = GRAPH_STATES.lock().unwrap();
	let graph_state = states.get(&graph_state_id);
	if graph_state.is_none() {
		return cx.throw_error("Graph state not found")
	}
	let graph_state = graph_state.unwrap();
	let graph_state = graph_state.lock().unwrap();
	let contains_user = graph_state.contains_user_graph(&dsnp_user_id);

	Ok(cx.boolean(contains_user))
}


/// Function to remove user graph from the graph state
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `graph_state_id` - Unique identifier for the graph state
/// * `dsnp_user_id` - DSNP user id
/// # Returns
/// * `JsResult<JsUndefined>` - Neon JsUndefined
/// # Errors
/// * Throws a Neon error if the graph state cannot be found
pub fn remove_user_graph(mut cx: FunctionContext) -> JsResult<JsUndefined> {
	let graph_state_id = cx.argument::<JsNumber>(0)?;
	let graph_state_id = graph_state_id.value(&mut cx) as usize;
	let dsnp_user_id = cx.argument::<JsNumber>(1)?;
	let dsnp_user_id = dsnp_user_id.value(&mut cx) as DsnpUserId;

	let mut states = GRAPH_STATES.lock().unwrap();
	let graph_state = states.get_mut(&graph_state_id);
	if graph_state.is_none() {
		return cx.throw_error("Graph state not found")
	}
	let graph_state = graph_state.unwrap();
	let mut graph_state = graph_state.lock().unwrap();
	graph_state.remove_user_graph(&dsnp_user_id);

	Ok(cx.undefined())
}

/// Function to free the graph state
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `graph_state_id` - Unique identifier for the graph state
/// # Returns
/// * `JsResult<JsUndefined>` - Neon JsUndefined
/// # Errors
/// * Throws a Neon error if the graph state cannot be found
pub fn free_graph_state(mut cx: FunctionContext) -> JsResult<JsUndefined> {
	let graph_state_id = cx.argument::<JsNumber>(0)?;
	let graph_state_id = graph_state_id.value(&mut cx) as usize;

	let mut states = GRAPH_STATES.lock().unwrap();
	let graph_state = states.remove(&graph_state_id);
	if graph_state.is_none() {
		return cx.throw_error("Graph state not found")
	}
	let graph_state = graph_state.unwrap();
	let graph_state = graph_state.lock().unwrap();
	drop(graph_state);

	Ok(cx.undefined())
}

#[neon::main]
fn main(mut cx: ModuleContext) -> NeonResult<()> {
	cx.export_function("printHelloGraph", print_hello_graph)?;
	cx.export_function("getGraphConfig", get_graph_config)?;
	cx.export_function("initializeGraphState", initialize_graph_state)?;
	cx.export_function("initializeGraphStateWithCapacity", initialize_graph_state_with_capacity)?;
	cx.export_function("getGraphCapacity", get_graph_capacity)?;
	cx.export_function("getGraphUsersCount", get_graph_users_count)?;
	cx.export_function("containsUserGraph", contains_user_graph)?;
	cx.export_function("removeUserGraph", remove_user_graph)?;
	cx.export_function("freeGraphState", free_graph_state)?;
	Ok(())
}
