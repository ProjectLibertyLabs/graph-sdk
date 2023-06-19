use crate::helper::*;
use dsnp_graph_config::{Config, DsnpUserId};
use dsnp_graph_core::{
	api::{
		api::{GraphAPI, GraphState},
		api_types::{Action, DsnpKeys, ImportBundle},
	},
	dsnp::dsnp_types::DsnpPublicKey,
};
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

	let environment = unsafe { environment_from_js(&mut cx, environment_obj) }?;
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
	let rust_environment = unsafe { environment_from_js(&mut cx, environment_obj) }?;
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
	let rust_environment = unsafe { environment_from_js(&mut cx, environment_obj) }?;
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

/// Get total count of graph states
/// # Arguments
/// * `cx` - Neon FunctionContext
/// # Returns
/// * `JsResult<JsNumber>` - Neon JsNumber containing the total count of graph states
/// # Errors
/// * Throws a Neon error
pub fn get_graph_states_count(mut cx: FunctionContext) -> JsResult<JsNumber> {
	let states = GRAPH_STATES.lock().unwrap();
	let states_count = states.len();

	Ok(cx.number(states_count as f64))
}

/// Get the capacity of the graph state
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `graph_state_id` - Unique identifier for the graph state
/// # Returns
/// * `JsResult<JsNumber>` - Neon JsNumber containing the capacity of the graph state
/// # Errors
/// * Throws a Neon error
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
/// * Throws a Neon error
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
/// * Throws a Neon error
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
/// * Throws a Neon error
pub fn remove_user_graph(mut cx: FunctionContext) -> JsResult<JsBoolean> {
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

	Ok(cx.boolean(true))
}

/// Function to import user data
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `graph_state_id` - Unique identifier for the graph state
/// * `payload` - JSON object for `ImportBundle`
/// # Returns
/// * `JsResult<JsUndefined>` - Neon JsUndefined
/// # Errors
/// * Throws a Neon error
pub fn import_user_data(mut cx: FunctionContext) -> JsResult<JsUndefined> {
	let graph_state_id = cx.argument::<JsNumber>(0)?;
	let graph_state_id = graph_state_id.value(&mut cx) as usize;
	let payload = cx.argument::<JsArray>(1)?;
	let rust_payload: Vec<ImportBundle> = import_bundle_from_js(&mut cx, payload);

	let mut states = GRAPH_STATES.lock().unwrap();
	let graph_state = states.get_mut(&graph_state_id);
	if graph_state.is_none() {
		return cx.throw_error("Graph state not found")
	}
	let graph_state = graph_state.unwrap();
	let mut graph_state = graph_state.lock().unwrap();

	let import_result = graph_state.import_users_data(&rust_payload);
	match import_result {
		Ok(_) => Ok(cx.undefined()),
		Err(e) => cx.throw_error(e.to_string()),
	}
}

/// Function to export graph updates
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `graph_state_id` - Unique identifier for the graph state
/// # Returns
/// * `JsResult<JsArray>` - Neon JsArray containing the exported updates
/// # Errors
/// * Throws a Neon error
pub fn export_graph_updates(mut cx: FunctionContext) -> JsResult<JsArray> {
	let graph_state_id = cx.argument::<JsNumber>(0)?;
	let graph_state_id = graph_state_id.value(&mut cx) as usize;

	let mut states = GRAPH_STATES.lock().unwrap();
	let graph_state = states.get_mut(&graph_state_id);
	if graph_state.is_none() {
		return cx.throw_error("Graph state not found")
	}
	let graph_state = graph_state.unwrap();
	let graph_state = graph_state.lock().unwrap();

	let updates = graph_state.export_updates();
	match updates {
		Ok(updates) => {
			let updates_js = updates_to_js(&mut cx, updates)?;
			Ok(updates_js)
		},
		Err(e) => cx.throw_error(e.to_string()),
	}
}

/// Function to get connections for user from the graph state (getConnectionsForUserGraph)
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `graph_state_id` - Unique identifier for the graph state
/// * `dsnp_user_id` - DSNP user id
/// # Returns
/// * `JsResult<JsArray>` - Neon JsArray containing the connections which is list of DSNPGraphEdge
/// # Errors
/// * Throws a Neon error
pub fn get_connections_for_user_graph(mut cx: FunctionContext) -> JsResult<JsArray> {
	let graph_state_id = cx.argument::<JsNumber>(0)?;
	let graph_state_id = graph_state_id.value(&mut cx) as usize;
	let dsnp_user_id = cx.argument::<JsNumber>(1)?;
	let dsnp_user_id = dsnp_user_id.value(&mut cx) as DsnpUserId;
	let schema_id = cx.argument::<JsNumber>(2)?;
	let schema_id = schema_id.value(&mut cx) as u16;
	let include_pending = cx.argument::<JsBoolean>(3)?;
	let include_pending = include_pending.value(&mut cx);
	let mut states = GRAPH_STATES.lock().unwrap();
	let graph_state = states.get_mut(&graph_state_id);
	if graph_state.is_none() {
		return cx.throw_error("Graph state not found")
	}
	let graph_state = graph_state.unwrap();
	let graph_state = graph_state.lock().unwrap();

	let connections =
		graph_state.get_connections_for_user_graph(&dsnp_user_id, &schema_id, include_pending);
	match connections {
		Ok(connections) => {
			let connections_js = connections_to_js(&mut cx, connections)?;
			Ok(connections_js)
		},
		Err(e) => cx.throw_error(e.to_string()),
	}
}

/// Function to applyActions to the graph state
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `graph_state_id` - Unique identifier for the graph state
/// * `actions` - JSArray containing the actions to apply
/// # Returns
/// * `JsResult<JsUndefined>` - Neon JsUndefined
/// # Errors
/// * Throws a Neon error
pub fn apply_actions(mut cx: FunctionContext) -> JsResult<JsUndefined> {
	let graph_state_id = cx.argument::<JsNumber>(0)?;
	let graph_state_id = graph_state_id.value(&mut cx) as usize;
	let actions = cx.argument::<JsArray>(1)?;
	let rust_actions: Vec<Action> = actions_from_js(&mut cx, actions);

	let mut states = GRAPH_STATES.lock().unwrap();
	let graph_state = states.get_mut(&graph_state_id);
	if graph_state.is_none() {
		return cx.throw_error("Graph state not found")
	}
	let graph_state = graph_state.unwrap();
	let mut graph_state = graph_state.lock().unwrap();

	let apply_result = graph_state.apply_actions(&rust_actions);
	match apply_result {
		Ok(_) => Ok(cx.undefined()),
		Err(e) => cx.throw_error(e.to_string()),
	}
}

/// Function to force calculate graphs for user
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `graph_state_id` - Unique identifier for the graph state
/// * `dsnp_user_id` - DSNP user id
/// # Returns
/// * `JsResult<JsObject>` - Neon JsObject containing the exported updates
/// # Errors
/// * Throws a Neon error
pub fn force_calculate_graphs(mut cx: FunctionContext) -> JsResult<JsArray> {
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
	let graph_state = graph_state.lock().unwrap();

	let update = graph_state.force_recalculate_graphs(&dsnp_user_id);
	match update {
		Ok(update) => {
			let update_js = updates_to_js(&mut cx, update)?;
			Ok(update_js)
		},
		Err(e) => cx.throw_error(e.to_string()),
	}
}

/// Function to get connections for user from the graph state (getConnectionsWithoutKeys)
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `graph_state_id` - Unique identifier for the graph state
/// # Returns
/// * `JsResult<JsArray>` - Neon JsArray containing the connections which is list of DSNPGraphEdge
/// # Errors
/// * Throws a Neon error
pub fn get_connections_without_keys(mut cx: FunctionContext) -> JsResult<JsArray> {
	let graph_state_id = cx.argument::<JsNumber>(0)?;
	let graph_state_id = graph_state_id.value(&mut cx) as usize;

	let mut states = GRAPH_STATES.lock().unwrap();
	let graph_state = states.get_mut(&graph_state_id);
	if graph_state.is_none() {
		return cx.throw_error("Graph state not found")
	}
	let graph_state = graph_state.unwrap();
	let graph_state = graph_state.lock().unwrap();

	let connections = graph_state.get_connections_without_keys();
	match connections {
		Ok(connections) => {
			let connections_js = cx.empty_array();
			for (i, connection) in connections.iter().enumerate() {
				let dsnp_user_id = cx.number(*connection as f64);
				connections_js.set(&mut cx, i as u32, dsnp_user_id)?;
			}
			Ok(connections_js)
		},
		Err(e) => cx.throw_error(e.to_string()),
	}
}

/// Function to get one sided private friendship connections for user from the graph state (getOneSidedPrivateFriendshipConnections)
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `graph_state_id` - Unique identifier for the graph state
/// * `dsnp_user_id` - DSNP user id
/// # Returns
/// * `JsResult<JsArray>` - Neon JsArray containing the connections which is list of DSNPGraphEdge
/// # Errors
/// * Throws a Neon error
pub fn get_one_sided_private_friendship_connections(mut cx: FunctionContext) -> JsResult<JsArray> {
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
	let graph_state = graph_state.lock().unwrap();

	let connections = graph_state.get_one_sided_private_friendship_connections(&dsnp_user_id);
	match connections {
		Ok(connections) => {
			let connections_js = connections_to_js(&mut cx, connections)?;
			Ok(connections_js)
		},
		Err(e) => cx.throw_error(e.to_string()),
	}
}

/// Function to get public keys for user from the graph state (getPublicKeys)
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `graph_state_id` - Unique identifier for the graph state
/// * `dsnp_user_id` - DSNP user id
/// # Returns
/// * `JsResult<JsArray>` - Neon JsArray containing the public keys which is list of DSNPGraphEdge
/// # Errors
/// * Throws a Neon error
pub fn get_public_keys(mut cx: FunctionContext) -> JsResult<JsArray> {
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
	let graph_state = graph_state.lock().unwrap();

	let public_keys = graph_state.get_public_keys(&dsnp_user_id);
	match public_keys {
		Ok(keys) => {
			let public_keys_js = public_keys_to_js(&mut cx, keys)?;
			Ok(public_keys_js)
		},
		Err(e) => cx.throw_error(e.to_string()),
	}
}

/// Function to deserialize DSNP keys
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `keys` - DSNP keys
/// # Returns
/// * `JsResult<JsArray>` - Neon JsArray containing the public keys which is list of DSNPGraphEdge
/// # Errors
/// * Throws a Neon error
pub fn deserialize_dsnp_keys(mut cx: FunctionContext) -> JsResult<JsArray> {
	let keys = cx.argument::<JsObject>(0)?;
	let try_keys_str = keys.to_string(&mut cx)?;
	let keys_str = try_keys_str.value(&mut cx);
	let rust_keys: DsnpKeys = DsnpKeys::try_from(keys_str.as_str()).unwrap();
	let deserialized_keys: Vec<DsnpPublicKey> =
		GraphState::deserialize_dsnp_keys(&rust_keys).unwrap_or_default();
	let keys_js = public_keys_to_js(&mut cx, deserialized_keys)?;

	Ok(keys_js)
}

/// Function to free the graph state
/// # Arguments
/// * `cx` - Neon FunctionContext
/// * `graph_state_id` - Unique identifier for the graph state
/// # Returns
/// * `JsResult<JsUndefined>` - Neon JsUndefined
/// # Errors
/// * Throws a Neon error
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
	cx.export_function("getGraphStatesCount", get_graph_states_count)?;
	cx.export_function("getGraphCapacity", get_graph_capacity)?;
	cx.export_function("getGraphUsersCount", get_graph_users_count)?;
	cx.export_function("containsUserGraph", contains_user_graph)?;
	cx.export_function("removeUserGraph", remove_user_graph)?;
	cx.export_function("importUserData", import_user_data)?;
	cx.export_function("exportUpdates", export_graph_updates)?;
	cx.export_function("getConnectionsForUserGraph", get_connections_for_user_graph)?;
	cx.export_function("applyActions", apply_actions)?;
	cx.export_function("forceCalculateGraphs", force_calculate_graphs)?;
	cx.export_function("getConnectionsWithoutKeys", get_connections_without_keys)?;
	cx.export_function(
		"getOneSidedPrivateFriendshipConnections",
		get_one_sided_private_friendship_connections,
	)?;
	cx.export_function("getPublicKeys", get_public_keys)?;
	cx.export_function("deserializeDsnpKeys", deserialize_dsnp_keys)?;
	cx.export_function("freeGraphState", free_graph_state)?;
	Ok(())
}
