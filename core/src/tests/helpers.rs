#![allow(dead_code)]

use crate::{
	dsnp::{
		api_types::*,
		compression::{CompressionBehavior, DeflateCompression},
		dsnp_types::*,
		schema::SchemaHandler,
	},
	graph::{graph::Graph, page::GraphPage},
	util::time::time_in_ksecs,
};
use std::{borrow::Borrow, cell::RefCell, collections::BTreeMap, rc::Rc};

use crate::{
	dsnp::{
		dsnp_configs::{DsnpVersionConfig, KeyPairType},
		reader_writer::DsnpWriter,
	},
	frequency::Frequency,
	graph::{
		key_manager::UserKeyManager,
		page::{PrivatePageDataProvider, PublicPageDataProvider},
		shared_state_manager::SharedStateManager,
	},
};
use base64::{engine::general_purpose, Engine as _};
use dryoc::keypair::StackKeyPair;
use dsnp_graph_config::{Environment, SchemaId};

pub fn create_graph_edge(id: &DsnpUserId) -> DsnpGraphEdge {
	DsnpGraphEdge { user_id: *id, since: time_in_ksecs() }
}

impl From<DsnpUserId> for DsnpPrid {
	fn from(id: DsnpUserId) -> Self {
		Self::from(id.to_le_bytes().to_vec())
	}
}

/// Create test data for a single page
pub fn create_test_ids_and_page() -> (Vec<DsnpUserId>, GraphPage) {
	let ids: Vec<DsnpUserId> = vec![1u64, 2u64, 3u64].to_vec();
	let pages = GraphPageBuilder::new(ConnectionType::Follow(PrivacyType::Private))
		.with_page(1, &ids, &vec![], 0)
		.build();
	let page = pages.first().expect("page should exist").clone();
	(ids, page)
}

/// Create a test instance of a Graph
pub fn create_test_graph() -> Graph {
	let mut page_builder = GraphPageBuilder::new(ConnectionType::Follow(PrivacyType::Private));
	let num_pages = 5;
	let ids_per_page = 5;
	let user_id = 3;
	let mut curr_id = 0u64;
	for i in 0..num_pages {
		let ids: Vec<DsnpUserId> = (curr_id..(curr_id + ids_per_page)).collect();
		page_builder = page_builder.with_page(i, &ids, &vec![], 0);
		curr_id += ids_per_page;
	}

	let env = Environment::Mainnet;
	let schema_id = env
		.get_config()
		.get_schema_id_from_connection_type(ConnectionType::Follow(PrivacyType::Private))
		.expect("should exist");
	let mut graph = Graph::new(
		env,
		user_id,
		schema_id,
		Rc::new(RefCell::from(UserKeyManager::new(
			user_id,
			Rc::new(RefCell::from(SharedStateManager::new())),
		))),
	);
	for p in page_builder.build() {
		let _ = graph.create_page(&p.page_id(), Some(p));
	}

	graph
}

pub const INNER_TEST_DATA: [DsnpGraphEdge; 24] = [
	DsnpGraphEdge { user_id: 4464346407956074433, since: 8764139209126768069 },
	DsnpGraphEdge { user_id: 6668873909761685247, since: 7188698398086794482 },
	DsnpGraphEdge { user_id: 3983583835435595748, since: 829969197675906694 },
	DsnpGraphEdge { user_id: 5786399658613658850, since: 1167130351887327801 },
	DsnpGraphEdge { user_id: 2550476024131609410, since: 3207336660582066677 },
	DsnpGraphEdge { user_id: 8998781204841458437, since: 6168655822672170066 },
	DsnpGraphEdge { user_id: 2295352874227852087, since: 8440514722944450399 },
	DsnpGraphEdge { user_id: 2614565340217427162, since: 1493098497079203084 },
	DsnpGraphEdge { user_id: 4565430723166717193, since: 524506678053007723 },
	DsnpGraphEdge { user_id: 5906091589969275177, since: 6902573244786247664 },
	DsnpGraphEdge { user_id: 7159305214820893538, since: 1936283288692888565 },
	DsnpGraphEdge { user_id: 8396161706254593904, since: 4536230715384416065 },
	DsnpGraphEdge { user_id: 8854381008488607807, since: 5159191892139543717 },
	DsnpGraphEdge { user_id: 73771519320842737, since: 2000265679509608646 },
	DsnpGraphEdge { user_id: 5927922952678211908, since: 7047213894547814807 },
	DsnpGraphEdge { user_id: 7267061036641634127, since: 5580380300958088425 },
	DsnpGraphEdge { user_id: 8662377975562298354, since: 9159136102447625539 },
	DsnpGraphEdge { user_id: 1567949913908946319, since: 4616269828673275240 },
	DsnpGraphEdge { user_id: 7106429197891368988, since: 1323323443768786584 },
	DsnpGraphEdge { user_id: 8402348483076003273, since: 8296993699355902565 },
	DsnpGraphEdge { user_id: 5584173321377371204, since: 1019201472789084023 },
	DsnpGraphEdge { user_id: 2998808192952224961, since: 8286911785053584720 },
	DsnpGraphEdge { user_id: 2554776608916995203, since: 7585826393836986397 },
	DsnpGraphEdge { user_id: 4944236923077661927, since: 5383633821359802131 },
];

pub fn avro_inner_payload() -> Vec<u8> {
	SchemaHandler::write_inner_graph(&INNER_TEST_DATA.to_vec()).unwrap()
}

pub fn avro_compressed_inner_payload() -> Vec<u8> {
	DeflateCompression::compress(&avro_inner_payload()).unwrap()
}

pub fn avro_public_payload() -> Vec<u8> {
	// encoded payload below matches INNER_TEST_DATA wrapped in a DsnpUserPublicGraphChunk
	let b64_payload = b"pgcBzgEx/jCCv5qI9dnF9HuKt5ehpIu9oPMB/tu5s5vfzIy5AeSzkPbhxrHDxwHI34PCjpnEyG6Mmuif0KXShBfE097BnvW1zaAB8oiR7eG3vbIghI2fqMqbjuVG6qenkePs34JZiqzL4ID1i+L5AaSxqMC18rubqwHujK/o49Hd2j++/ffo9trdouoBtJOqm+HU5shImNSa0YPLxrgpkpSO0LrD1dt+1sXbuu6CtscO0qTT8MvA0/ajAeCfzLnu8u7KvwHEjYKAqYr72sYB6s+E8YzZh9814L3XoMmllIXpAYKdopun8/bzff7A7c6rp4rh9QHKsuywn+yQmY8B4ofHusqzi4YCjOOaubTCr8I3iIWopJOhm8SkAa7mwYvh7N3MwwGendqh1ODk2ckB0vON46mUv/GaAeSP9/jzt/m28AGG1cW9wfDkm/4BnpatwvWIvcIr0JXk1u+8pJCAAbjgu5yH7Y2fxQGw+sfMpeqx3SSS18zMsNuRm+kByomanO3z66TmAYjRyo6Wg/z+mgHu3a/C3ZX3pByCs5anuIr0nVOg9e+RmpeDgeYBhvLL78fysfRGuoD9xr26osbSAc7T0Nal0rqdiQGmnPCu+pvBtpUBAA==";
	general_purpose::STANDARD.decode(b64_payload).unwrap()
}

pub struct KeyDataBuilder {
	key_pairs: Vec<GraphKeyPair>,
}

impl KeyDataBuilder {
	pub fn new() -> Self {
		KeyDataBuilder { key_pairs: vec![] }
	}

	pub fn with_key_pairs(mut self, key_pairs: &[GraphKeyPair]) -> Self {
		self.key_pairs.extend_from_slice(key_pairs);
		self
	}

	pub fn get_key_pairs(&self) -> &Vec<GraphKeyPair> {
		&self.key_pairs
	}

	pub fn build(self) -> Vec<KeyData> {
		self.key_pairs
			.iter()
			.enumerate()
			.map(|(i, pair)| KeyData {
				index: i as u16,
				content: Frequency::write_public_key(&DsnpPublicKey {
					key_id: Some(i as u64),
					key: pair.public_key.to_vec(),
				})
				.expect("should serialize"),
			})
			.collect()
	}
}

pub struct GraphPageBuilder {
	connection_type: ConnectionType,
	// using BTreeMap to keep the pages sorted
	pages: BTreeMap<PageId, (Vec<DsnpUserId>, Vec<DsnpPrid>, u32)>,
}

impl GraphPageBuilder {
	pub fn new(connection_type: ConnectionType) -> Self {
		Self { connection_type, pages: BTreeMap::new() }
	}

	pub fn with_page(
		mut self,
		page_id: PageId,
		connections: &[DsnpUserId],
		prids: &[DsnpPrid],
		content_hash: u32,
	) -> Self {
		let (c, p, hash) = self.pages.entry(page_id).or_insert((vec![], vec![], 0));
		c.extend_from_slice(connections);
		p.extend_from_slice(prids);
		*hash = content_hash;
		self
	}

	pub fn build(self) -> Vec<GraphPage> {
		self.pages
			.iter()
			.map(|(page_id, (connections, prids, hash))| {
				let mut page = GraphPage::new(self.connection_type.privacy_type(), *page_id);
				page.set_connections(
					connections.iter().map(|c| DsnpGraphEdge { user_id: *c, since: 0 }).collect(),
				);
				if self.connection_type == ConnectionType::Friendship(PrivacyType::Private) {
					page.set_prids(prids.clone()).expect("should set");
				}
				page.set_content_hash(*hash);
				page
			})
			.collect()
	}
}

pub struct PageDataBuilder {
	connection_type: ConnectionType,
	page_builder: GraphPageBuilder,
	resolved_key: ResolvedKeyPair,
}

impl PageDataBuilder {
	pub fn new(connection_type: ConnectionType) -> Self {
		Self {
			connection_type,
			page_builder: GraphPageBuilder::new(connection_type),
			resolved_key: ResolvedKeyPair {
				key_pair: KeyPairType::Version1_0(StackKeyPair::gen()),
				key_id: 1,
			},
		}
	}

	pub fn with_page(
		mut self,
		page_id: PageId,
		connections: &[DsnpUserId],
		prids: &[DsnpPrid],
		content_hash: u32,
	) -> Self {
		self.page_builder = self.page_builder.with_page(page_id, connections, prids, content_hash);
		self
	}

	pub fn with_encryption_key(mut self, key_bundle: ResolvedKeyPair) -> Self {
		self.resolved_key = key_bundle;
		self
	}

	pub fn build(self) -> Vec<PageData> {
		let dsnp_config: DsnpVersionConfig = self.resolved_key.key_pair.borrow().into();
		self.page_builder
			.build()
			.iter()
			.map(|page| match self.connection_type.privacy_type() {
				PrivacyType::Public =>
					page.to_public_page_data().expect("should write public page"),
				PrivacyType::Private =>
					page.to_private_page_data(&dsnp_config, &self.resolved_key).unwrap(),
			})
			.collect()
	}
}

pub struct ImportBundleBuilder {
	env: Environment,
	dsnp_user_id: DsnpUserId,
	schema_id: SchemaId,
	key_builder: KeyDataBuilder,
	page_data_builder: PageDataBuilder,
}

impl ImportBundleBuilder {
	pub fn new(env: Environment, dsnp_user_id: DsnpUserId, schema_id: SchemaId) -> Self {
		let connection_type =
			env.get_config().get_connection_type_from_schema_id(schema_id).unwrap();
		ImportBundleBuilder {
			env,
			dsnp_user_id,
			schema_id,
			key_builder: KeyDataBuilder::new(),
			page_data_builder: PageDataBuilder::new(connection_type),
		}
	}

	pub fn with_page(
		mut self,
		page_id: PageId,
		connections: &[DsnpUserId],
		prids: &[DsnpPrid],
		content_hash: u32,
	) -> Self {
		self.page_data_builder =
			self.page_data_builder.with_page(page_id, connections, prids, content_hash);
		self
	}

	pub fn with_encryption_key(mut self, key_bundle: ResolvedKeyPair) -> Self {
		self.page_data_builder = self.page_data_builder.with_encryption_key(key_bundle);
		self
	}

	pub fn with_key_pairs(mut self, key_pairs: &[GraphKeyPair]) -> Self {
		self.key_builder = self.key_builder.with_key_pairs(key_pairs);
		self
	}

	pub fn build(self) -> ImportBundle {
		let key_pairs = self.key_builder.get_key_pairs().clone();
		let pages: Vec<PageData> = self.page_data_builder.build();
		let keys: Vec<KeyData> = self.key_builder.build();

		ImportBundle {
			dsnp_keys: DsnpKeys { keys, keys_hash: 232, dsnp_user_id: self.dsnp_user_id },
			dsnp_user_id: self.dsnp_user_id,
			schema_id: self.schema_id,
			key_pairs,
			pages,
		}
	}
}
