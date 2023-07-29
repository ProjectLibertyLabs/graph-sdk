//! Builders used mostly in integration and unit tests
use crate::{
	api::api_types::{
		DsnpKeys, GraphKeyPair, ImportBundle, KeyData, PageData, PageId, ResolvedKeyPair, Update,
	},
	dsnp::{
		dsnp_configs::{DsnpVersionConfig, KeyPairType},
		dsnp_types::{DsnpGraphEdge, DsnpPrid, DsnpPublicKey, DsnpUserId},
		reader_writer::DsnpWriter,
	},
	frequency::Frequency,
	graph::page::{GraphPage, PrivatePageDataProvider, PublicPageDataProvider},
};
use dryoc::keypair::StackKeyPair;
use dsnp_graph_config::{ConnectionType, Environment, GraphKeyType, PrivacyType, SchemaId};
use std::collections::BTreeMap;

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

	pub fn with_generated_key(mut self) -> Self {
		let raw_key_pair = StackKeyPair::gen();
		self.key_pairs.extend_from_slice(&vec![GraphKeyPair {
			key_type: GraphKeyType::X25519,
			secret_key: raw_key_pair.secret_key.to_vec(),
			public_key: raw_key_pair.public_key.to_vec(),
		}]);
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
	pages: BTreeMap<PageId, (Vec<DsnpGraphEdge>, Vec<DsnpPrid>, u32)>,
}

impl GraphPageBuilder {
	pub fn new(connection_type: ConnectionType) -> Self {
		Self { connection_type, pages: BTreeMap::new() }
	}

	pub fn with_page(
		mut self,
		page_id: PageId,
		connections: &[(DsnpUserId, u64)],
		prids: &[DsnpPrid],
		content_hash: u32,
	) -> Self {
		let (c, p, hash) = self.pages.entry(page_id).or_insert((vec![], vec![], 0));
		let edges: Vec<_> = connections
			.iter()
			.map(|(u, s)| DsnpGraphEdge { user_id: *u, since: *s })
			.collect();
		c.extend_from_slice(&edges);
		p.extend_from_slice(prids);
		*hash = content_hash;
		self
	}

	pub fn build(&self) -> Vec<GraphPage> {
		self.pages
			.iter()
			.map(|(page_id, (connections, prids, hash))| {
				let mut page = GraphPage::new(self.connection_type.privacy_type(), *page_id);
				page.set_connections(connections.clone());
				if self.connection_type == ConnectionType::Friendship(PrivacyType::Private) {
					page.unchecked_set_prids(prids.clone());
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
	use_noisy_creation_time: bool,
}

impl PageDataBuilder {
	pub fn new(connection_type: ConnectionType) -> Self {
		Self {
			connection_type,
			page_builder: GraphPageBuilder::new(connection_type),
			resolved_key: ResolvedKeyPair {
				key_pair: KeyPairType::Version1_0(StackKeyPair::gen()),
				key_id: 0,
			},
			use_noisy_creation_time: false,
		}
	}

	pub fn with_page(
		mut self,
		page_id: PageId,
		connections: &[(DsnpUserId, u64)],
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

	pub fn with_noisy_creation_time(mut self, b: bool) -> Self {
		self.use_noisy_creation_time = b;
		self
	}

	pub fn build(self) -> Vec<PageData> {
		let dsnp_config: DsnpVersionConfig = (&self.resolved_key.key_pair).into();
		self.page_builder
			.build()
			.iter()
			.map(|page| match self.connection_type.privacy_type() {
				PrivacyType::Public =>
					page.to_public_page_data().expect("should write public page"),
				PrivacyType::Private => page
					.to_private_page_data(&dsnp_config, &self.resolved_key)
					.expect("should write private page"),
			})
			.collect()
	}

	pub fn build_with_size(&self) -> Vec<(usize, PageData)> {
		let dsnp_config: DsnpVersionConfig = (&self.resolved_key.key_pair).into();
		self.page_builder
			.build()
			.iter()
			.map(|page| match self.connection_type.privacy_type() {
				PrivacyType::Public => (
					page.connections().len(),
					page.to_public_page_data().expect("should write public page"),
				),
				PrivacyType::Private => (
					page.connections().len(),
					page.to_private_page_data(&dsnp_config, &self.resolved_key)
						.expect("should write private page"),
				),
			})
			.collect()
	}
}

pub struct ImportBundleBuilder {
	_env: Environment,
	dsnp_user_id: DsnpUserId,
	schema_id: SchemaId,
	key_builder: KeyDataBuilder,
	page_data_builder: PageDataBuilder,
}

impl ImportBundleBuilder {
	pub fn new(env: Environment, dsnp_user_id: DsnpUserId, schema_id: SchemaId) -> Self {
		let connection_type = env
			.get_config()
			.get_connection_type_from_schema_id(schema_id)
			.unwrap_or(ConnectionType::Follow(PrivacyType::Public));
		ImportBundleBuilder {
			_env: env,
			dsnp_user_id,
			schema_id,
			key_builder: KeyDataBuilder::new(),
			page_data_builder: PageDataBuilder::new(connection_type),
		}
	}

	pub fn with_page(
		mut self,
		page_id: PageId,
		connections: &[(DsnpUserId, u64)],
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
		let keys_hash = match keys.is_empty() {
			true => 0,
			false => 232,
		};

		ImportBundle {
			dsnp_keys: match keys.len() {
				0 => None,
				_ => Some(DsnpKeys { keys, keys_hash, dsnp_user_id: self.dsnp_user_id }),
			},
			dsnp_user_id: self.dsnp_user_id,
			schema_id: self.schema_id,
			key_pairs,
			pages,
		}
	}

	pub fn build_from(original: &ImportBundle, updates: &[Update]) -> ImportBundle {
		let mut new_bundle = original.clone();
		for u in updates {
			match u {
				Update::PersistPage {
					page_id,
					schema_id,
					owner_dsnp_user_id,
					payload,
					prev_hash,
				} => {
					if *owner_dsnp_user_id != new_bundle.dsnp_user_id ||
						*schema_id != new_bundle.schema_id
					{
						continue
					}
					let new_page =
						PageData { content_hash: 1, content: payload.clone(), page_id: *page_id };
					match original.pages.iter().position(|p| p.page_id == *page_id) {
						Some(ind) => {
							let old_page = original.pages.get(ind).unwrap();
							assert_eq!(old_page.content_hash, *prev_hash);
							new_bundle.pages.remove(ind);
							new_bundle.pages.insert(ind, new_page);
						},
						None => {
							new_bundle.pages.push(new_page);
						},
					}
				},
				Update::DeletePage { page_id, prev_hash, schema_id, owner_dsnp_user_id } => {
					if *owner_dsnp_user_id != new_bundle.dsnp_user_id ||
						*schema_id != new_bundle.schema_id
					{
						continue
					}
					let ind = original
						.pages
						.iter()
						.position(|p| p.page_id == *page_id)
						.expect("Page should exist!");
					let old_page = original.pages.get(ind).unwrap();
					assert_eq!(old_page.content_hash, *prev_hash);
					new_bundle.pages.remove(ind);
				},
				Update::AddKey { prev_hash, payload, owner_dsnp_user_id } => {
					if *owner_dsnp_user_id != new_bundle.dsnp_user_id {
						continue
					}
					assert_eq!(
						match &original.dsnp_keys {
							Some(dsnp_keys) => dsnp_keys.keys_hash,
							None => 0,
						},
						*prev_hash
					);
					match new_bundle.dsnp_keys.iter_mut().next() {
						Some(dsnp_keys) => {
							dsnp_keys.keys_hash = 1;
							dsnp_keys.keys.push(KeyData {
								content: payload.clone(),
								index: dsnp_keys.keys.len() as u16,
							});
						},
						None =>
							new_bundle.dsnp_keys = Some(DsnpKeys {
								dsnp_user_id: new_bundle.dsnp_user_id,
								keys_hash: 1,
								keys: vec![KeyData { content: payload.clone(), index: 0u16 }],
							}),
					};
				},
			}
		}
		new_bundle
	}
}
