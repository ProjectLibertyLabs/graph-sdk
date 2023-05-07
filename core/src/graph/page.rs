#![allow(dead_code)]
use crate::{
	dsnp::{api_types::*, dsnp_types::*},
	util::time::time_in_ksecs,
};
use anyhow::{Error, Result};
use std::borrow::Borrow;

use crate::{
	dsnp::{
		dsnp_configs::DsnpVersionConfig,
		reader_writer::{DsnpReader, DsnpWriter},
		schema::SchemaHandler,
	},
	frequency::Frequency,
	types::PrivateGraphChunk,
};

/// A traits that returns a removed page binary payload according to the DSNP Graph schema
pub trait RemovedPageDataProvider {
	fn to_removed_page_data(&self) -> PageData;
}

/// A traits that returns a public page binary payload according to the DSNP Public Graph schema
pub trait PublicPageDataProvider {
	fn to_public_page_data(&self) -> Result<PageData>;
}

/// A traits that returns a private page binary payload according to the DSNP Private Graph schema
pub trait PrivatePageDataProvider {
	fn to_private_page_data(
		&self,
		dsnp_version_config: &DsnpVersionConfig,
		key: &ResolvedKeyPair,
	) -> Result<PageData>;
}

/// Graph page structure
#[derive(Debug, Clone, PartialEq)]
pub struct GraphPage {
	/// Page ID
	page_id: PageId,
	/// Privacy type of owning graph
	privacy_type: PrivacyType,
	/// Current content hash of page as retrieved from chain
	content_hash: u32,
	/// List of PRIds
	prids: Vec<DsnpPrid>,
	/// List of connections
	connections: Vec<DsnpGraphEdge>,
}

/// Conversion for Public Graph
impl TryFrom<&PageData> for GraphPage {
	type Error = Error;
	fn try_from(PageData { content_hash, content, page_id }: &PageData) -> Result<Self> {
		Ok(Self {
			page_id: *page_id,
			privacy_type: PrivacyType::Public,
			content_hash: *content_hash,
			prids: Vec::new(),
			connections: Frequency::read_public_graph(&content)?,
		})
	}
}

/// Conversion for Private Graph
impl TryFrom<(&PageData, &DsnpVersionConfig, &Vec<ResolvedKeyPair>)> for GraphPage {
	type Error = Error;
	fn try_from(
		(PageData { content_hash, content, page_id }, dsnp_version_config, keys): (
			&PageData,
			&DsnpVersionConfig,
			&Vec<ResolvedKeyPair>,
		),
	) -> Result<Self> {
		let mut private_graph_chunk: Option<PrivateGraphChunk> = None;

		// read key_id from page
		let DsnpUserPrivateGraphChunk { key_id, .. } =
			SchemaHandler::read_private_graph_chunk(&content)?;

		// First try the key that was indicated in the page
		if let Some(indicated_key) = keys.iter().find(|k| k.key_id == key_id) {
			let secret_key = indicated_key.key_pair.clone().into();
			if let Ok(chunk) =
				Frequency::read_private_graph(&content, &dsnp_version_config, &secret_key)
			{
				private_graph_chunk = Some(chunk);
			}
		}

		if private_graph_chunk.is_none() {
			// could not decrypt using the indicated key id ,let try with other keys
			for other_key in keys.iter().filter(|k| k.key_id != key_id) {
				let secret_key = other_key.key_pair.clone().into();
				if let Ok(chunk) =
					Frequency::read_private_graph(&content, &dsnp_version_config, &secret_key)
				{
					private_graph_chunk = Some(chunk);
					break
				}
			}
		}

		match private_graph_chunk {
			None => Err(Error::msg("Unable to decrypt private graph chunk with any existing keys")),
			Some(chunk) => Ok(GraphPage {
				page_id: *page_id,
				privacy_type: PrivacyType::Private,
				content_hash: *content_hash,
				prids: chunk.prids,
				connections: chunk.inner_graph,
			}),
		}
	}
}

impl RemovedPageDataProvider for GraphPage {
	fn to_removed_page_data(&self) -> PageData {
		PageData { content_hash: self.content_hash, page_id: self.page_id, content: Vec::new() }
	}
}

impl PublicPageDataProvider for GraphPage {
	fn to_public_page_data(&self) -> Result<PageData> {
		if self.privacy_type != PrivacyType::Public {
			return Err(Error::msg("Incompatible privacy type for blob export"))
		}

		Ok(PageData {
			content_hash: self.content_hash,
			page_id: self.page_id,
			content: Frequency::write_public_graph(self.connections())?,
		})
	}
}

impl PrivatePageDataProvider for GraphPage {
	fn to_private_page_data(
		&self,
		dsnp_version_config: &DsnpVersionConfig,
		key: &ResolvedKeyPair,
	) -> Result<PageData> {
		if self.privacy_type != PrivacyType::Private {
			return Err(Error::msg("Incompatible privacy type for blob export"))
		}

		Ok(PageData {
			page_id: self.page_id,
			content_hash: self.content_hash,
			content: Frequency::write_private_graph(
				&PrivateGraphChunk {
					prids: self.prids.clone(),
					inner_graph: self.connections.clone(),
					key_id: key.clone().key_id,
				},
				dsnp_version_config,
				&key.key_pair.borrow().into(),
			)?,
		})
	}
}

impl GraphPage {
	/// Create a new, empty page
	pub fn new(privacy_type: PrivacyType, page_id: PageId) -> Self {
		Self {
			page_id,
			privacy_type,
			content_hash: 0,
			prids: Vec::<DsnpPrid>::new(),
			connections: Vec::<DsnpGraphEdge>::new(),
		}
	}

	/// Getter for the prids in the page
	pub fn prids(&self) -> &Vec<DsnpPrid> {
		&self.prids
	}

	/// Getter for the connections in the page
	pub fn connections(&self) -> &Vec<DsnpGraphEdge> {
		&self.connections
	}

	/// Setter for the connections in the page
	pub fn set_connections(&mut self, connections: Vec<DsnpGraphEdge>) {
		self.connections = connections
	}

	/// Getter for the content hash
	pub fn content_hash(&self) -> u32 {
		self.content_hash
	}

	/// Setter for the content hash
	pub fn set_content_hash(&mut self, content_hash: u32) {
		self.content_hash = content_hash;
	}

	/// Get page id
	pub fn page_id(&self) -> PageId {
		self.page_id
	}

	/// Tester to check if the page contains a connection to a particular DsnpUserId
	pub fn contains(&self, connection_id: &DsnpUserId) -> bool {
		self.connections.iter().any(|c| c.user_id == *connection_id)
	}

	/// Checks if any of the users contains in this pages connections
	pub fn contains_any(&self, connections: &Vec<DsnpUserId>) -> bool {
		self.connections.iter().map(|c| c.user_id).any(|id| connections.contains(&id))
	}

	/// Function to test if the page is empty
	pub fn is_empty(&self) -> bool {
		self.connections.is_empty()
	}

	/// Add a connection to the page. Fail if the connection is already present.
	pub fn add_connection(&mut self, connection_id: &DsnpUserId) -> Result<()> {
		if self.contains(connection_id) {
			return Err(Error::msg("Add of duplicate connection detected"))
		}

		self.connections
			.push(DsnpGraphEdge { user_id: *connection_id, since: time_in_ksecs() });
		Ok(())
	}

	/// Remove a connection from the page. Error if connection not found in page.
	pub fn remove_connection(&mut self, connection_id: &DsnpUserId) -> Result<()> {
		if !self.contains(connection_id) {
			return Err(Error::msg("Connection not found in page"))
		}

		self.connections.retain(|c| c.user_id != *connection_id);
		Ok(())
	}

	/// Remove all connections in the list from the page. It is not an error if none of the connections are present.
	pub fn remove_connections(&mut self, ids: &Vec<DsnpUserId>) {
		self.connections.retain(|c| !ids.contains(&c.user_id));
	}

	/// Refresh PRIds based on latest
	pub fn set_prids(&mut self, prids: Vec<DsnpPrid>) -> Result<()> {
		if self.connections.len() != prids.len() {
			return Err(Error::msg(format!(
				"page_id: {}, prids len should be equal to connections len (connections: {}, prids: {})",
				self.page_id,
				self.connections.len(),
				prids.len()
			)))
		}
		self.prids = prids;
		Ok(())
	}

	/// Clear prids in the page
	pub fn clear_prids(&mut self) {
		self.prids = vec![];
	}
}

#[cfg(all(test, not(feature = "calculate-page-capacity")))]
mod test {
	use super::*;
	use crate::{dsnp::dsnp_configs::KeyPairType, tests::helpers::*};
	use dryoc::keypair::StackKeyPair;
	use dsnp_graph_config::{
		ConnectionType::{Follow, Friendship},
		DsnpVersion,
		PrivacyType::Public,
	};
	#[allow(unused_imports)]
	use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};

	#[test]
	fn new_page() {
		let page = GraphPage::new(PrivacyType::Private, 0);

		assert_eq!(page.is_empty(), true, "Page should be empty");
	}

	#[test]
	fn graph_page_getters_setters() {
		let mut page = GraphPage::new(PrivacyType::Private, 0);
		let prids: Vec<DsnpPrid> = vec![1, 2, 3, 4].iter().map(|id| DsnpPrid::from(*id)).collect();
		let connections: Vec<DsnpGraphEdge> =
			vec![5, 6, 7, 8].iter().map(create_graph_edge).collect();

		page.set_connections(connections.clone());
		assert!(page.set_prids(prids.clone()).is_ok());
		assert_eq!(&prids, page.prids());
		assert_eq!(&connections, page.connections());
		assert_eq!(0, page.page_id());
	}

	#[test]
	fn page_contains_finds_item() {
		let (ids, page) = create_test_ids_and_page();
		for (id, _) in ids {
			assert_eq!(page.contains(&id as &DsnpUserId), true);
		}
	}

	#[test]
	fn page_contains_does_not_find_missing_items() {
		let (_, page) = create_test_ids_and_page();
		assert_eq!(page.contains(&(4 as DsnpUserId)), false);
	}

	#[test]
	fn page_contains_any_finds_none() {
		let ids_to_find = vec![100, 200, 300, 400, 500];
		let (_, page) = create_test_ids_and_page();
		assert_eq!(page.contains_any(&ids_to_find), false);
	}

	#[test]
	fn page_contains_any_finds_some() {
		let (ids, page) = create_test_ids_and_page();
		let (user, _) = ids.first().unwrap();
		let ids_to_find = vec![*user, 100, 200, 300, 400, 500];
		assert_eq!(page.contains_any(&ids_to_find), true);
	}

	#[test]
	fn is_empty_on_nonempty_page_returns_false() {
		let (_, page) = create_test_ids_and_page();
		assert_eq!(page.is_empty(), false);
	}

	#[test]
	fn add_duplicate_connection_fails() {
		let (_, mut page) = create_test_ids_and_page();
		assert_eq!(page.add_connection(&1u64).is_err(), true);
	}

	#[test]
	fn add_connection_succeeds() {
		let id: DsnpUserId = 1;
		let mut page = GraphPage::new(PrivacyType::Private, 0);

		assert_eq!(page.add_connection(&id).is_ok(), true);
		assert_eq!(page.contains(&id), true);
	}

	#[test]
	fn remove_connection_not_found_fails() {
		let (_, mut page) = create_test_ids_and_page();

		assert_eq!(page.remove_connection(&4u64).is_err(), true);
	}

	#[test]
	fn remove_connection_succeeds() {
		let (_, mut page) = create_test_ids_and_page();
		let id_to_remove = 1u64;

		assert_eq!(page.remove_connection(&id_to_remove).is_ok(), true);
		assert_eq!(page.contains(&id_to_remove), false);
	}

	#[test]
	fn remove_list_of_connections_removes_matching_connections() {
		let (ids, mut page) = create_test_ids_and_page();
		let mut ids_to_remove: Vec<DsnpUserId> =
			ids.iter().map(|(c, _)| c).take(ids.len() / 2).cloned().collect();
		ids_to_remove.extend_from_slice(vec![100, 200, 300, 400].as_slice());

		page.remove_connections(&ids_to_remove);
		for id in ids_to_remove {
			assert_eq!(page.contains(&id), false);
		}
	}

	#[test]
	fn update_prids_with_wrong_size_should_fail() {
		// arrange
		let (ids, mut page) = create_test_ids_and_page();
		let mut prids: Vec<DsnpPrid> =
			ids.iter().map(|(id, _)| DsnpPrid::new(&id.to_le_bytes())).collect();
		prids.remove(0); // making prids size different than connection size

		// act
		let res = page.set_prids(prids);

		// assert
		assert!(res.is_err())
	}

	#[test]
	fn graph_page_public_try_from_page_data_should_work_correctly() {
		// arrange
		let page_id = 10;
		let privacy_type = Public;
		let content_hash = 20;
		let connections = vec![(1, 0), (2, 0), (3, 0), (4, 0)];
		let page_data = PageDataBuilder::new(Follow(privacy_type))
			.with_page(page_id, &connections, &vec![], content_hash)
			.build();
		let expected = GraphPage {
			page_id,
			privacy_type,
			content_hash,
			prids: vec![],
			connections: connections
				.iter()
				.map(|(c, s)| DsnpGraphEdge { user_id: *c, since: *s })
				.collect(),
		};
		// act
		let graph_page = GraphPage::try_from(page_data.get(0).unwrap());

		// assert
		assert!(graph_page.is_ok());
		let graph_page = graph_page.unwrap();
		assert_eq!(graph_page, expected);
	}

	#[test]
	fn graph_page_private_follow_try_from_page_data_should_work_correctly() {
		// arrange
		let page_id = 10;
		let privacy_type = PrivacyType::Private;
		let content_hash = 20;
		let dsnp = DsnpVersionConfig::new(DsnpVersion::Version1_0);
		let connections = vec![(1, 0), (2, 0), (3, 0), (4, 0)];
		let key =
			ResolvedKeyPair { key_id: 1, key_pair: KeyPairType::Version1_0(StackKeyPair::gen()) };
		let page_data = PageDataBuilder::new(Follow(privacy_type))
			.with_page(page_id, &connections, &vec![], content_hash)
			.with_encryption_key(key.clone())
			.build();
		let expected = GraphPage {
			page_id,
			privacy_type,
			content_hash,
			prids: vec![],
			connections: connections
				.iter()
				.map(|(c, s)| DsnpGraphEdge { user_id: *c, since: *s })
				.collect(),
		};

		// act
		let graph_page = GraphPage::try_from((page_data.get(0).unwrap(), &dsnp, &vec![key]));

		// assert
		assert!(graph_page.is_ok());
		let graph_page = graph_page.unwrap();
		assert_eq!(graph_page, expected);
	}

	#[test]
	fn graph_page_private_friendship_try_from_page_data_should_work_correctly() {
		// arrange
		let page_id = 10;
		let privacy_type = PrivacyType::Private;
		let content_hash = 20;
		let dsnp = DsnpVersionConfig::new(DsnpVersion::Version1_0);
		let connections = vec![(1, 0), (2, 0), (3, 0), (4, 0)];
		let prids: Vec<DsnpPrid> = connections.iter().map(|(id, _)| DsnpPrid::from(*id)).collect();
		let key =
			ResolvedKeyPair { key_id: 1, key_pair: KeyPairType::Version1_0(StackKeyPair::gen()) };
		let page_data = PageDataBuilder::new(Friendship(privacy_type))
			.with_page(page_id, &connections, &prids, content_hash)
			.with_encryption_key(key.clone())
			.build();
		let expected = GraphPage {
			page_id,
			privacy_type,
			content_hash,
			prids,
			connections: connections
				.iter()
				.map(|(c, s)| DsnpGraphEdge { user_id: *c, since: *s })
				.collect(),
		};

		// act
		let graph_page = GraphPage::try_from((page_data.get(0).unwrap(), &dsnp, &vec![key]));

		// assert
		assert!(graph_page.is_ok());
		let graph_page = graph_page.unwrap();
		assert_eq!(graph_page, expected);
	}

	#[test]
	fn graph_page_private_try_from_page_data_should_try_other_keys_to_decrypt() {
		// arrange
		let page_id = 10;
		let privacy_type = PrivacyType::Private;
		let content_hash = 20;
		let dsnp = DsnpVersionConfig::new(DsnpVersion::Version1_0);
		let connections = vec![(1, 0), (2, 0), (3, 0), (4, 0)];
		let prids: Vec<DsnpPrid> = connections.iter().map(|(id, _)| DsnpPrid::from(*id)).collect();
		let key =
			ResolvedKeyPair { key_id: 1, key_pair: KeyPairType::Version1_0(StackKeyPair::gen()) };
		let other_key =
			ResolvedKeyPair { key_id: 2, key_pair: KeyPairType::Version1_0(StackKeyPair::gen()) };
		let page_data = PageDataBuilder::new(Friendship(privacy_type))
			.with_page(page_id, &connections, &prids, content_hash)
			.with_encryption_key(ResolvedKeyPair {
				key_id: 1,
				key_pair: other_key.key_pair.clone(),
			})
			.build();

		let expected = GraphPage {
			page_id,
			privacy_type,
			content_hash,
			prids,
			connections: connections
				.iter()
				.map(|(c, s)| DsnpGraphEdge { user_id: *c, since: *s })
				.collect(),
		};

		// act
		let graph_page =
			GraphPage::try_from((page_data.get(0).unwrap(), &dsnp, &vec![key, other_key]));

		// assert
		assert!(graph_page.is_ok());
		let graph_page = graph_page.unwrap();
		assert_eq!(graph_page, expected);
	}

	#[test]
	fn graph_page_private_try_from_page_data_with_wrong_keys_should_fail() {
		// arrange
		let page_id = 10;
		let privacy_type = PrivacyType::Private;
		let content_hash = 20;
		let dsnp = DsnpVersionConfig::new(DsnpVersion::Version1_0);
		let connections = vec![(1, 0), (2, 0), (3, 0), (4, 0)];
		let prids: Vec<DsnpPrid> = connections.iter().map(|(id, _)| DsnpPrid::from(*id)).collect();
		let encrypted_key =
			ResolvedKeyPair { key_id: 1, key_pair: KeyPairType::Version1_0(StackKeyPair::gen()) };
		let other_key =
			ResolvedKeyPair { key_id: 2, key_pair: KeyPairType::Version1_0(StackKeyPair::gen()) };
		let page_data = PageDataBuilder::new(Friendship(privacy_type))
			.with_page(page_id, &connections, &prids, content_hash)
			.with_encryption_key(encrypted_key)
			.build();

		// act
		let graph_page = GraphPage::try_from((page_data.get(0).unwrap(), &dsnp, &vec![other_key]));

		// assert
		assert!(graph_page.is_err());
	}

	#[test]
	fn removed_page_data_provider_should_return_removed_page_as_expected() {
		// arrange
		let graph = GraphPage {
			page_id: 1,
			privacy_type: PrivacyType::Private,
			content_hash: 10,
			prids: vec![DsnpPrid::from(vec![1u8, 2, 3, 4, 5, 6, 7, 8])],
			connections: vec![DsnpGraphEdge { user_id: 70, since: 2873 }],
		};
		let expected = PageData { page_id: 1, content: vec![], content_hash: 10 };

		// act
		let removed = graph.to_removed_page_data();

		// assert
		assert_eq!(expected, removed);
	}

	#[test]
	fn public_page_data_provider_should_return_public_page_as_expected() {
		// arrange
		let page_id = 10;
		let privacy_type = Public;
		let content_hash = 20;
		let connections = vec![(1, 0), (2, 0), (3, 0), (4, 0)];
		let page_data = PageDataBuilder::new(Follow(privacy_type))
			.with_page(page_id, &connections, &vec![], content_hash)
			.build();
		let graph = GraphPage {
			page_id,
			privacy_type,
			content_hash,
			prids: vec![],
			connections: connections
				.iter()
				.map(|(c, s)| DsnpGraphEdge { user_id: *c, since: *s })
				.collect(),
		};

		// act
		let public = graph.to_public_page_data();

		// assert
		assert!(public.is_ok());
		let public = public.unwrap();
		assert_eq!(&public, page_data.get(0).unwrap());
	}

	#[test]
	fn private_page_data_provider_should_return_private_page_as_expected() {
		// arrange
		let page_id = 10;
		let privacy_type = PrivacyType::Private;
		let content_hash = 20;
		let dsnp = DsnpVersionConfig::new(DsnpVersion::Version1_0);
		let connections = vec![(1, 0), (2, 0), (3, 0), (4, 0)];
		let prids: Vec<DsnpPrid> = connections.iter().map(|(id, _)| DsnpPrid::from(*id)).collect();
		let key =
			ResolvedKeyPair { key_id: 1, key_pair: KeyPairType::Version1_0(StackKeyPair::gen()) };
		let page_data = PageDataBuilder::new(Friendship(privacy_type))
			.with_page(page_id, &connections, &prids, content_hash)
			.with_encryption_key(key.clone())
			.build();
		let graph_page =
			GraphPage::try_from((page_data.get(0).unwrap(), &dsnp, &vec![key.clone()]))
				.expect("should work");

		// act
		let private = graph_page.to_private_page_data(&dsnp, &key).expect("should work");
		let graph_page2 = GraphPage::try_from((&private, &dsnp, &vec![key.clone()]));

		// assert
		assert!(graph_page2.is_ok());
		let graph_page2 = graph_page2.unwrap();
		assert_eq!(graph_page, graph_page2);
	}
}

#[cfg(all(test, feature = "calculate-page-capacity"))]
mod page_capacity {
	use dsnp_graph_config::ALL_CONNECTION_TYPES;

	use super::*;
	use crate::tests::helpers::*;
	use std::{collections::hash_map::HashMap, path::PathBuf};

	#[test]
	fn calculate_page_capacities() {
		let mut capacity_map: HashMap<ConnectionType, usize> = HashMap::new();

		for c in ALL_CONNECTION_TYPES {
			let mut result_vec: Vec<usize> = Vec::new();
			for _ in 0..1000 {
				result_vec.push(benchmark_page_capacity(c).0);
			}
			result_vec.sort();
			capacity_map.insert(c, *result_vec.first().unwrap());
		}

		let code = format!(
			"use dsnp_graph_config::{{ConnectionType, ConnectionType::Follow, ConnectionType::Friendship, PrivacyType::Public, PrivacyType::Private}};
use lazy_static::lazy_static;
use std::collections::hash_map::*;

lazy_static! {{
	pub static ref PAGE_CAPACITIY_MAP: HashMap<ConnectionType, usize> = {{
		let m = HashMap::from({:?});
		m
	}};
}}
",
			capacity_map.iter().collect::<Vec::<(&ConnectionType, &usize)>>()
		);

		let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
		path.push("src/graph/page_capacities.rs");

		let result = std::fs::write(path, code);
		if result.is_err() {
			println!("Error: {:?}", result);
		}

		assert!(true);
	}
}
