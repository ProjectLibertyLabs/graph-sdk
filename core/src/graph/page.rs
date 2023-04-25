use crate::{
	dsnp::{api_types::*, dsnp_types::*},
	util::time::time_in_ksecs,
};
use anyhow::{Error, Result};
use std::borrow::Borrow;

use crate::dsnp::{
	compression::{CompressionBehavior, DeflateCompression},
	dsnp_configs::DsnpVersionConfig,
	encryption::EncryptionBehavior,
	schema::SchemaHandler,
};

const APPROX_MAX_CONNECTIONS_PER_PAGE: usize = 10; // todo: determine best size for this

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
	fn try_from(PageData { content_hash, content, page_id, .. }: &PageData) -> Result<Self> {
		let DsnpUserPublicGraphChunk { compressed_public_graph } =
			DsnpUserPublicGraphChunk::try_from(content.as_slice())?;
		let uncompressed_chunk = DeflateCompression::decompress(&compressed_public_graph)?;
		Ok(Self {
			page_id: *page_id,
			privacy_type: PrivacyType::Public,
			content_hash: *content_hash,
			prids: Vec::new(),
			connections: SchemaHandler::read_inner_graph(&uncompressed_chunk)?,
		})
	}
}

/// Conversion for Private Graph
impl TryFrom<(&PageData, &DsnpVersionConfig, &Vec<ResolvedKeyPair>)> for GraphPage {
	type Error = Error;
	fn try_from(
		(PageData { content_hash, content, page_id, .. }, dsnp_version_config, keys): (
			&PageData,
			&DsnpVersionConfig,
			&Vec<ResolvedKeyPair>,
		),
	) -> Result<Self> {
		let DsnpUserPrivateGraphChunk { key_id, encrypted_compressed_private_graph, prids } =
			DsnpUserPrivateGraphChunk::try_from(content.as_slice())?;
		let mut decrypted_chunk: Option<Vec<u8>> = None;

		let algorithm = dsnp_version_config.get_algorithm();
		// First try the key that was indicated in the page
		if let Some(indicated_key) = keys.iter().find(|k| k.key_id == key_id) {
			if let Ok(data) = algorithm.decrypt(
				&encrypted_compressed_private_graph,
				&indicated_key.key_pair.clone().into(),
			) {
				decrypted_chunk = Some(data);
			}
		}

		// If we couldn't decrypt with (or find) the indicated key, try all keys
		decrypted_chunk = match decrypted_chunk {
			Some(_) => decrypted_chunk,
			None => keys.iter().find_map(|k| {
				algorithm
					.decrypt(&encrypted_compressed_private_graph, &k.key_pair.clone().into())
					.ok()
			}),
		};

		match decrypted_chunk {
			None => Err(Error::msg("Unable to decrypt private graph chunk with any existing keys")),
			Some(data) => {
				let uncompressed_chunk = DeflateCompression::decompress(&data)?;
				let connections = SchemaHandler::read_inner_graph(&uncompressed_chunk)?;

				Ok(GraphPage {
					page_id: *page_id,
					privacy_type: PrivacyType::Private,
					content_hash: *content_hash,
					prids,
					connections,
				})
			},
		}
	}
}

impl TryFrom<GraphPage> for DsnpUserPublicGraphChunk {
	type Error = Error;
	fn try_from(data: GraphPage) -> Result<Self, Self::Error> {
		let uncompressed_public_graph = SchemaHandler::write_inner_graph(&data.connections)?;
		let compressed_public_graph = DeflateCompression::compress(&uncompressed_public_graph)?;
		Ok(Self { compressed_public_graph })
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

	/// Setter for the prids in the page
	pub fn set_prids(&mut self, prids: Vec<DsnpPrid>) {
		self.prids = prids;
	}

	/// Setter for the connections in the page
	pub fn set_connections(&mut self, connections: Vec<DsnpGraphEdge>) {
		self.connections = connections
	}

	/// Get page id
	pub fn page_id(&self) -> PageId {
		self.page_id
	}

	/// Tester to check if the page contains a connection to a particular DsnpUserId
	pub fn contains(&self, connection_id: &DsnpUserId) -> bool {
		self.connections.iter().any(|c| c.user_id == *connection_id)
	}

	pub fn contains_any(&self, connections: &Vec<DsnpUserId>) -> bool {
		self.connections.iter().map(|c| c.user_id).any(|id| connections.contains(&id))
	}

	/// Function to test if the page is empty
	pub fn is_empty(&self) -> bool {
		self.connections.is_empty()
	}

	// Determine if page is full
	// 	aggressive:false -> use a simple heuristic based on the number of connections
	//  aggressive:true  -> do actual compression to determine resulting actual page size
	pub fn is_full(&self, aggressive: bool) -> bool {
		if !aggressive {
			return self.connections.len() >= APPROX_MAX_CONNECTIONS_PER_PAGE
		}

		todo!()
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

	// Remove all connections in the list from the page. It is not an error if none of the connections are present.
	pub fn remove_connections(&mut self, ids: &Vec<DsnpUserId>) {
		self.connections.retain(|c| !ids.contains(&c.user_id));
	}

	/// Refresh PRIds based on latest keys
	pub fn update_prids<E: EncryptionBehavior>(
		&mut self,
		_prid_keys: &Vec<DsnpKeys>,
	) -> Result<()> {
		todo!()
	}

	// TODO: make trait-based
	// Convert to a binary payload according to the DSNP Public Graph schema
	pub fn to_removed_page_data(&self) -> PageData {
		PageData { content_hash: self.content_hash, page_id: self.page_id, content: Vec::new() }
	}

	// TODO: make trait-based
	// Convert to a binary payload according to the DSNP Public Graph schema
	pub fn to_public_page_data(&self) -> Result<PageData> {
		if self.privacy_type != PrivacyType::Public {
			return Err(Error::msg("Incompatible privacy type for blob export"))
		}

		let content =
			SchemaHandler::write_public_graph_chunk(&self.connections.clone().try_into()?)?;
		Ok(PageData { content_hash: self.content_hash, page_id: self.page_id, content })
	}

	// TODO: make trait-based
	/// Convert to an encrypted binary payload according to the DSNP Private Graph schema
	pub fn to_private_page_data(
		&self,
		dsnp_version_config: &DsnpVersionConfig,
		key: &ResolvedKeyPair,
		_prids: &Vec<DsnpPrid>,
	) -> Result<PageData> {
		if self.privacy_type != PrivacyType::Private {
			return Err(Error::msg("Incompatible privacy type for blob export"))
		}

		// TODO should update PRI's before calling this
		// self.update_prids::<E>(prid_keys)?;

		let DsnpUserPublicGraphChunk { compressed_public_graph } =
			self.connections.clone().try_into()?;
		let private_chunk = DsnpUserPrivateGraphChunk {
			key_id: key.clone().key_id,
			prids: self.prids.clone(),
			encrypted_compressed_private_graph: dsnp_version_config
				.get_algorithm()
				.encrypt(&compressed_public_graph, &key.key_pair.borrow().into())?,
		};

		Ok(PageData {
			page_id: self.page_id,
			content_hash: self.content_hash,
			content: SchemaHandler::write_private_graph_chunk(&private_chunk)?,
		})
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::tests::helpers::*;
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

		page.set_prids(prids.clone());
		page.set_connections(connections.clone());
		assert_eq!(&prids, page.prids());
		assert_eq!(&connections, page.connections());
		assert_eq!(0, page.page_id());
	}

	#[test]
	fn page_contains_finds_item() {
		let (ids, page) = create_test_ids_and_page();
		for id in ids {
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
		let ids_to_find = vec![*ids.first().unwrap(), 100, 200, 300, 400, 500];
		assert_eq!(page.contains_any(&ids_to_find), true);
	}

	#[test]
	fn is_empty_on_nonempty_page_returns_false() {
		let (_, page) = create_test_ids_and_page();
		assert_eq!(page.is_empty(), false);
	}

	#[test]
	fn is_full_non_aggressive_returns_false_for_non_full() {
		let mut page = GraphPage::new(PrivacyType::Private, 0);
		let mut last_connection: DsnpUserId = 0;
		while page.connections.len() < APPROX_MAX_CONNECTIONS_PER_PAGE {
			assert_eq!(page.is_full(false), false);
			let _ = page.add_connection(&last_connection);
			last_connection += 1;
		}
	}

	#[test]
	fn is_full_non_aggressive_returns_true_for_full() {
		let connections = (0..APPROX_MAX_CONNECTIONS_PER_PAGE as u64).collect::<Vec<u64>>();
		let pages = GraphPageBuilder::new(ConnectionType::Follow(PrivacyType::Private))
			.with_page(1, &connections, &vec![])
			.build();

		let page = pages.first().expect("page should exist");
		assert_eq!(page.is_full(false), true);
	}

	#[test]
	#[ignore = "todo"]
	fn is_full_aggressive_returns_false_for_non_full() {}

	#[test]
	#[ignore = "todo"]
	fn is_full_aggressive_returns_true_for_full() {}

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
		let mut ids_to_remove: Vec<DsnpUserId> = ids.iter().take(ids.len() / 2).cloned().collect();
		ids_to_remove.extend_from_slice(vec![100, 200, 300, 400].as_slice());

		page.remove_connections(&ids_to_remove);
		for id in ids_to_remove {
			assert_eq!(page.contains(&id), false);
		}
	}

	#[test]
	#[ignore = "todo"]
	fn update_prids_with_bad_key_fails() {}
}
