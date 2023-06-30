//! Builder utility to help creating a new Config
//!
#![allow(dead_code)]
use crate::{Config, DsnpVersion, SchemaConfig, SchemaId};
use std::collections::HashMap;

pub struct ConfigBuilder {
	sdk_max_stale_friendship_days: u32,
	max_graph_page_size_bytes: u32,
	max_page_id: u32,
	max_key_page_size_bytes: u32,
	schema_map: HashMap<SchemaId, SchemaConfig>,
	dsnp_versions: Vec<DsnpVersion>,
}

impl ConfigBuilder {
	pub fn new() -> Self {
		Self {
			schema_map: HashMap::new(),
			max_graph_page_size_bytes: 1024,
			max_page_id: 16,
			max_key_page_size_bytes: 65536,
			sdk_max_stale_friendship_days: 90,
			dsnp_versions: vec![],
		}
	}

	pub fn with_sdk_max_stale_friendship_days(
		mut self,
		sdk_max_stale_friendship_days: u32,
	) -> Self {
		self.sdk_max_stale_friendship_days = sdk_max_stale_friendship_days;
		self
	}

	pub fn with_max_page_id(mut self, max_page_id: u32) -> Self {
		self.max_page_id = max_page_id;
		self
	}

	pub fn with_max_key_page_size_bytes(mut self, max_key_page_size_bytes: u32) -> Self {
		self.max_key_page_size_bytes = max_key_page_size_bytes;
		self
	}

	pub fn with_max_graph_page_size_bytes(mut self, max_graph_page_size_bytes: u32) -> Self {
		self.max_graph_page_size_bytes = max_graph_page_size_bytes;
		self
	}

	pub fn with_schema(mut self, schema_id: SchemaId, config: SchemaConfig) -> Self {
		self.schema_map.insert(schema_id, config);
		self
	}

	pub fn build(self) -> Config {
		Config {
			sdk_max_stale_friendship_days: self.sdk_max_stale_friendship_days,
			schema_map: self.schema_map,
			max_page_id: self.max_page_id,
			max_key_page_size_bytes: self.max_key_page_size_bytes,
			max_graph_page_size_bytes: self.max_graph_page_size_bytes,
			dsnp_versions: self.dsnp_versions,
		}
	}
}
