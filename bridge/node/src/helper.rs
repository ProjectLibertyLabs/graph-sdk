use dsnp_graph_config::{Config as RustConfig, DsnpVersion, MAINNET_CONFIG, ROCOCO_CONFIG};
use neon::{
	context::Context, handle::Handle, prelude::FunctionContext, result::JsResult, types::JsObject,
};
use std::collections::HashMap;

// TODO implement neon helpers for these types
