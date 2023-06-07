use dsnp_graph_config::{errors::DsnpGraphError, SchemaId};
use dsnp_graph_core::{
	api::api::{GraphAPI, GraphState},
	dsnp::dsnp_types::DsnpUserId,
};
use neon::prelude::*;
use std::{panic, sync::Mutex};

// Collection of GraphStates
#[allow(clippy::vec_box)]
static GRAPH_STATES: Mutex<Vec<Box<GraphState>>> = Mutex::new(Vec::new());

// Neon implementation of print_hello_graph function
fn print_hello_graph(mut cx: FunctionContext) -> JsResult<JsUndefined> {
	println!("Hello, Graph!");
	Ok(cx.undefined())
}

//TODO implement
