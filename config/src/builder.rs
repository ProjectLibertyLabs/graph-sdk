#![allow(dead_code)]
use crate::{Config, DsnpVersionConfig, SchemaConfig, SchemaId};
use std::collections::HashMap;

pub struct ConfigBuilder {
	max_graph_page_size_bytes: u32,
	max_page_id: u32,
	max_key_page_size_bytes: u32,
	schema_map: HashMap<SchemaId, SchemaConfig>,
	dsnp_version_map: HashMap<String, DsnpVersionConfig>,
}

impl ConfigBuilder {
	pub fn new() -> Self {
		Self {
			dsnp_version_map: HashMap::new(),
			schema_map: HashMap::new(),
			max_graph_page_size_bytes: 1024,
			max_page_id: 16,
			max_key_page_size_bytes: 65536,
		}
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

	pub fn with_dsnp_version(mut self, dsnp_version: String, config: DsnpVersionConfig) -> Self {
		self.dsnp_version_map.insert(dsnp_version, config);
		self
	}

	pub fn build(self) -> Config {
		Config {
			schema_map: self.schema_map,
			dsnp_version_map: self.dsnp_version_map,
			max_page_id: self.max_page_id,
			max_key_page_size_bytes: self.max_key_page_size_bytes,
			max_graph_page_size_bytes: self.max_graph_page_size_bytes,
		}
	}
}
