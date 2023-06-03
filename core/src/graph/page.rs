#![allow(dead_code)]
use crate::{
	dsnp::{api_types::*, dsnp_types::*},
	util::time::time_in_ksecs,
};
use dsnp_graph_config::errors::{DsnpGraphError, DsnpGraphResult};
use std::borrow::Borrow;

use crate::{
	dsnp::{
		dsnp_configs::DsnpVersionConfig,
		reader_writer::{DsnpReader, DsnpWriter},
		schema::SchemaHandler,
	},
	frequency::Frequency,
	util::{transactional_hashmap::Transactional, transactional_vec::TransactionalVec},
};

/// A traits that returns a removed page binary payload according to the DSNP Graph schema
pub trait RemovedPageDataProvider {
	fn to_removed_page_data(&self) -> PageData;
}

/// A traits that returns a public page binary payload according to the DSNP Public Graph schema
pub trait PublicPageDataProvider {
	fn to_public_page_data(&self) -> DsnpGraphResult<PageData>;
}

/// A traits that returns a private page binary payload according to the DSNP Private Graph schema
pub trait PrivatePageDataProvider {
	fn to_private_page_data(
		&self,
		dsnp_version_config: &DsnpVersionConfig,
		key: &ResolvedKeyPair,
	) -> DsnpGraphResult<PageData>;
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
	prids: TransactionalVec<DsnpPrid>,
	/// List of connections
	connections: TransactionalVec<DsnpGraphEdge>,
}

/// Conversion for Public Graph
impl TryFrom<&PageData> for GraphPage {
	type Error = DsnpGraphError;

	fn try_from(PageData { content_hash, content, page_id }: &PageData) -> DsnpGraphResult<Self> {
		Ok(Self {
			page_id: *page_id,
			privacy_type: PrivacyType::Public,
			content_hash: *content_hash,
			prids: TransactionalVec::new(),
			connections: TransactionalVec::from(Frequency::read_public_graph(&content)?),
		})
	}
}

/// Conversion for Private Graph
impl TryFrom<(&PageData, &DsnpVersionConfig, &Vec<ResolvedKeyPair>)> for GraphPage {
	type Error = DsnpGraphError;

	fn try_from(
		(PageData { content_hash, content, page_id }, dsnp_version_config, keys): (
			&PageData,
			&DsnpVersionConfig,
			&Vec<ResolvedKeyPair>,
		),
	) -> DsnpGraphResult<Self> {
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
			// could not decrypt using the indicated key id ,lets try with other keys
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
			None => Err(DsnpGraphError::UnableToDecryptGraphChunkWithAnyKey),
			Some(chunk) => Ok(GraphPage {
				page_id: *page_id,
				privacy_type: PrivacyType::Private,
				content_hash: *content_hash,
				prids: TransactionalVec::from(chunk.prids),
				connections: TransactionalVec::from(chunk.inner_graph),
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
	fn to_public_page_data(&self) -> DsnpGraphResult<PageData> {
		if self.privacy_type != PrivacyType::Public {
			return Err(DsnpGraphError::IncompatiblePrivacyTypeForBlobExport)
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
	) -> DsnpGraphResult<PageData> {
		if self.privacy_type != PrivacyType::Private {
			return Err(DsnpGraphError::IncompatiblePrivacyTypeForBlobExport)
		}

		Ok(PageData {
			page_id: self.page_id,
			content_hash: self.content_hash,
			content: Frequency::write_private_graph(
				&PrivateGraphChunk {
					prids: self.prids.inner().clone(),
					inner_graph: self.connections.inner().clone(),
					key_id: key.clone().key_id,
				},
				dsnp_version_config,
				&key.key_pair.borrow().into(),
			)?,
		})
	}
}

/// Allows transactional operation support for graph page
impl Transactional for GraphPage {
	fn commit(&mut self) {
		self.prids.commit();
		self.connections.commit();
	}

	fn rollback(&mut self) {
		self.prids.rollback();
		self.connections.rollback();
	}
}

impl GraphPage {
	/// Create a new, empty page
	pub fn new(privacy_type: PrivacyType, page_id: PageId) -> Self {
		Self {
			page_id,
			privacy_type,
			content_hash: 0,
			prids: TransactionalVec::<DsnpPrid>::new(),
			connections: TransactionalVec::<DsnpGraphEdge>::new(),
		}
	}

	/// Getter for the prids in the page
	pub fn prids(&self) -> &Vec<DsnpPrid> {
		self.prids.inner()
	}

	/// Getter for the connections in the page
	pub fn connections(&self) -> &Vec<DsnpGraphEdge> {
		self.connections.inner()
	}

	/// Setter for the connections in the page
	pub fn set_connections(&mut self, connections: Vec<DsnpGraphEdge>) {
		self.connections.clear();
		self.connections.extend_from_slice(&connections);
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
		self.connections.inner().iter().any(|c| c.user_id == *connection_id)
	}

	/// Checks if any of the users contains in this pages connections
	pub fn contains_any(&self, connections: &Vec<DsnpUserId>) -> bool {
		self.connections
			.inner()
			.iter()
			.map(|c| c.user_id)
			.any(|id| connections.contains(&id))
	}

	/// Function to test if the page is empty
	pub fn is_empty(&self) -> bool {
		self.connections.inner().is_empty()
	}

	/// Add a connection to the page. Fail if the connection is already present.
	pub fn add_connection(&mut self, connection_id: &DsnpUserId) -> DsnpGraphResult<()> {
		if self.contains(connection_id) {
			return Err(DsnpGraphError::DuplicateConnectionDetected)
		}

		self.connections
			.push(DsnpGraphEdge { user_id: *connection_id, since: time_in_ksecs() });
		Ok(())
	}

	/// Remove a connection from the page. Error if connection not found in page.
	pub fn remove_connection(&mut self, connection_id: &DsnpUserId) -> DsnpGraphResult<()> {
		if !self.contains(connection_id) {
			return Err(DsnpGraphError::ConnectionNotFound)
		}

		self.connections.retain(|c| c.user_id != *connection_id);
		Ok(())
	}

	/// Remove all connections in the list from the page. It is not an error if none of the connections are present.
	pub fn remove_connections(&mut self, ids: &Vec<DsnpUserId>) {
		self.connections.retain(|c| !ids.contains(&c.user_id));
	}

	/// Refresh PRIds based on latest
	pub fn set_prids(&mut self, prids: Vec<DsnpPrid>) -> DsnpGraphResult<()> {
		if self.connections.len() != prids.len() {
			return Err(DsnpGraphError::PridsLenShouldBeEqualToConnectionsLen(
				self.page_id,
				self.connections.len(),
				prids.len(),
			))
		}
		self.prids.clear();
		self.prids.extend_from_slice(&prids);
		Ok(())
	}

	/// verifies that the size of prids should be the same as connection in private friendship
	pub fn verify_prid_len(&self, connection_type: ConnectionType) -> DsnpGraphResult<()> {
		if connection_type == ConnectionType::Friendship(PrivacyType::Private) &&
			self.connections.len() != self.prids.len()
		{
			return Err(DsnpGraphError::PridsLenShouldBeEqualToConnectionsLen(
				self.page_id,
				self.connections.len(),
				self.prids.len(),
			))
		}
		Ok(())
	}

	/// unchecked sets prids (mostly used in tests)
	pub fn unchecked_set_prids(&mut self, prids: Vec<DsnpPrid>) {
		self.prids.clear();
		self.prids.extend_from_slice(&prids);
	}

	/// Clear prids in the page
	pub fn clear_prids(&mut self) {
		self.prids.clear();
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::{
		dsnp::dsnp_configs::KeyPairType, tests::helpers::*, util::builders::PageDataBuilder,
	};
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
			prids: TransactionalVec::new(),
			connections: TransactionalVec::from(
				connections
					.iter()
					.map(|(c, s)| DsnpGraphEdge { user_id: *c, since: *s })
					.collect(),
			),
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
			prids: TransactionalVec::new(),
			connections: TransactionalVec::from(
				connections
					.iter()
					.map(|(c, s)| DsnpGraphEdge { user_id: *c, since: *s })
					.collect(),
			),
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
			prids: TransactionalVec::from(prids),
			connections: TransactionalVec::from(
				connections
					.iter()
					.map(|(c, s)| DsnpGraphEdge { user_id: *c, since: *s })
					.collect(),
			),
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
			prids: TransactionalVec::from(prids),
			connections: TransactionalVec::from(
				connections
					.iter()
					.map(|(c, s)| DsnpGraphEdge { user_id: *c, since: *s })
					.collect(),
			),
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
			prids: TransactionalVec::from(vec![DsnpPrid::from(vec![1u8, 2, 3, 4, 5, 6, 7, 8])]),
			connections: TransactionalVec::from(vec![DsnpGraphEdge { user_id: 70, since: 2873 }]),
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
			prids: TransactionalVec::new(),
			connections: TransactionalVec::from(
				connections
					.iter()
					.map(|(c, s)| DsnpGraphEdge { user_id: *c, since: *s })
					.collect(),
			),
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

	#[test]
	fn graph_page_commit_should_persist_changes_on_page() {
		// arrange
		let connections = vec![(1, 0), (2, 0), (3, 0), (4, 0)];
		let prids: Vec<DsnpPrid> = connections.iter().map(|(id, _)| DsnpPrid::from(*id)).collect();
		let mut page = GraphPage::new(PrivacyType::Private, 1);
		connections.iter().for_each(|(u, _)| {
			page.add_connection(u).unwrap();
		});
		page.set_prids(prids).unwrap();
		let prid_len = page.prids.len();
		let connection_len = page.connections.len();

		// act
		page.commit();
		page.rollback();

		// assert
		assert_eq!(prid_len, page.prids.len());
		assert_eq!(connection_len, page.connections.len());
	}

	#[test]
	fn graph_page_rollback_should_revert_changes_on_page() {
		// arrange
		let prid = DsnpPrid::from(vec![1u8, 2, 3, 4, 5, 6, 7, 8]);
		let connection = DsnpGraphEdge { user_id: 70, since: 2873 };
		let mut page = GraphPage {
			page_id: 1,
			privacy_type: PrivacyType::Private,
			content_hash: 10,
			prids: TransactionalVec::from(vec![prid.clone()]),
			connections: TransactionalVec::from(vec![connection]),
		};
		page.add_connection(&10).expect("should add");
		page.set_prids(vec![prid.clone(), DsnpPrid::from(vec![10u8, 20, 30, 40, 50, 60, 70, 80])])
			.expect("should add prid");

		// act
		page.rollback();

		// assert
		assert_eq!(page.prids.inner(), &vec![prid]);
		assert_eq!(page.connections.inner(), &vec![connection]);
	}
}
