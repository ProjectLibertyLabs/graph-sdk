#![allow(dead_code)] // todo: remove
use crate::{
	dsnp::{api_types::*, dsnp_types::*},
	graph::updates::{UpdateEvent, UpdateTracker},
	util::time::time_in_ksecs,
};
use anyhow::{Error, Result};
use std::collections::{HashMap, HashSet};

use super::{
	compression::{CompressionBehavior, DeflateCompression},
	encryption::EncryptionBehavior,
	schema::SchemaHandler,
};

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
impl TryFrom<&PageBlob> for GraphPage {
	type Error = Error;
	fn try_from(PageBlob { content_hash, payload, page_id, .. }: &PageBlob) -> Result<Self> {
		let DsnpUserPublicGraphChunk { compressed_public_graph } =
			DsnpUserPublicGraphChunk::try_from(payload.as_slice())?;
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
impl<E: EncryptionBehavior> TryFrom<(&PageBlob, &Vec<KeyPair<E>>)> for GraphPage {
	type Error = Error;
	fn try_from(
		(PageBlob { content_hash, payload, page_id, .. }, keys): (&PageBlob, &Vec<KeyPair<E>>),
	) -> Result<Self> {
		let DsnpUserPrivateGraphChunk { key_id, encrypted_compressed_private_graph, prids } =
			DsnpUserPrivateGraphChunk::try_from(payload.as_slice())?;
		let mut decrypted_chunk: Option<Vec<u8>> = None;

		// First try the key that was indicated in the page
		if let Some(indicated_key) = keys.iter().find(|k| k.key_id == key_id) {
			if let Ok(data) =
				E::decrypt(&encrypted_compressed_private_graph, &indicated_key.private_key)
			{
				decrypted_chunk = Some(data);
			}
		}

		// If we couldn't decrypt with (or find) the indicated key, try all keys
		decrypted_chunk = match decrypted_chunk {
			Some(_) => decrypted_chunk,
			None => keys
				.iter()
				.find_map(|k| E::decrypt(&encrypted_compressed_private_graph, &k.private_key).ok()),
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

type PageMap = HashMap<PageId, GraphPage>;
type GraphMap = HashMap<ConnectionType, Graph>;
/// Graph structure to hold pages of connections of a single type
#[derive(Debug, Clone)]
pub struct Graph {
	pub connection_type: ConnectionType,
	pages: PageMap,
}

/// Structure to hold all of a User's Graphs, mapped by ConnectionType
#[derive(Debug, Clone)]
pub struct UserGraph {
	user_id: DsnpUserId,
	graphs: GraphMap,
	pub update_tracker: UpdateTracker,
}

impl UserGraph {
	/// Create a new, empty UserGraph
	pub fn new(user_id: &DsnpUserId) -> Self {
		Self {
			user_id: *user_id,
			graphs: GraphMap::from([
				(
					ConnectionType::Follow(PrivacyType::Public),
					Graph::new(ConnectionType::Follow(PrivacyType::Public)),
				),
				(
					ConnectionType::Follow(PrivacyType::Private),
					Graph::new(ConnectionType::Follow(PrivacyType::Private)),
				),
				(
					ConnectionType::Friendship(PrivacyType::Public),
					Graph::new(ConnectionType::Friendship(PrivacyType::Public)),
				),
				(
					ConnectionType::Friendship(PrivacyType::Private),
					Graph::new(ConnectionType::Friendship(PrivacyType::Private)),
				),
			]),
			update_tracker: UpdateTracker::new(),
		}
	}

	/// Getter for map of graphs
	pub fn graphs(&self) -> &GraphMap {
		&self.graphs
	}

	/// Getter for UpdateTracker
	pub fn update_tracker(&mut self) -> &mut UpdateTracker {
		&mut self.update_tracker
	}

	/// Getter for the user's graph for the specified ConnectionType
	pub fn graph(&self, connection_type: &ConnectionType) -> &Graph {
		self.graphs.get(connection_type).expect("UserGraph local instance is corrupt")
	}

	/// Mutable getter for the user's graph for the specified ConnectionType
	pub fn graph_mut(&mut self, connection_type: &ConnectionType) -> &mut Graph {
		self.graphs
			.get_mut(connection_type)
			.expect("UserGraph local instance is corrupt")
	}

	/// Setter for the specified graph connection type
	pub fn set_graph(&mut self, connection_type: &ConnectionType, graph: Graph) {
		self.graphs.insert(*connection_type, graph);
	}

	/// Clear the specified graph type for this user
	pub fn clear_graph(&mut self, connection_type: &ConnectionType) {
		if let Some(g) = self.graphs.get_mut(connection_type) {
			g.clear();
		}
	}

	/// Clear all graphs associated with this user
	pub fn clear_all(&mut self) {
		self.graphs.iter_mut().for_each(|(_, g)| g.clear());
	}

	/// Cacluate pending updates
	pub fn calculate_updates<E: EncryptionBehavior>(
		&mut self,
		connection_keys: &Vec<DsnpKeys<E>>,
		encryption_key: (u64, &PublicKey<E>),
	) -> Result<Vec<ExportBundle>> {
		self.graphs
			.iter()
			.map(|(connection_type, graph)| {
				graph.calculate_updates(
					self.update_tracker.get_updates_for_connection_type(*connection_type),
					&self.user_id,
					connection_keys,
					encryption_key,
				)
			})
			.collect()
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
		const MAX_CONNECTIONS_PER_PAGE: usize = 10; // todo: determine best size for this
		if !aggressive {
			return self.connections.len() >= MAX_CONNECTIONS_PER_PAGE
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

	// Remove all connections in the list from the page. Error if none of the connections are present in the page.
	pub fn remove_connections(&mut self, ids: &Vec<DsnpUserId>) {
		self.connections.retain(|c| !ids.contains(&c.user_id));
	}

	/// Refresh PRIds based on latest keys
	pub fn update_prids<E: EncryptionBehavior>(
		&mut self,
		_prid_keys: &Vec<DsnpKeys<E>>,
	) -> Result<()> {
		todo!()
	}

	// TODO: make trait-based
	// Convert to a binary payload according to the DSNP Public Graph schema
	pub fn to_public_blob(&self) -> Result<PageBlob> {
		if self.privacy_type != PrivacyType::Public {
			return Err(Error::msg("Incompatible privacy type for blob export"))
		}

		let payload =
			SchemaHandler::write_public_graph_chunk(&self.connections.clone().try_into()?)?;
		Ok(PageBlob { content_hash: self.content_hash, page_id: self.page_id, payload })
	}

	// TODO: make trait-based
	/// Convert to an encrypted binary payload according to the DSNP Private Graph schema
	pub fn to_private_blob<E: EncryptionBehavior>(
		&mut self,
		(key_id, key): (u64, &PublicKey<E>),
		prid_keys: &Vec<DsnpKeys<E>>,
	) -> Result<PageBlob> {
		if self.privacy_type != PrivacyType::Private {
			return Err(Error::msg("Incompatible privacy type for blob export"))
		}

		self.update_prids(prid_keys)?;

		let DsnpUserPublicGraphChunk { compressed_public_graph } =
			self.connections.clone().try_into()?;
		let private_chunk = DsnpUserPrivateGraphChunk {
			key_id,
			prids: self.prids.clone(),
			encrypted_compressed_private_graph: E::encrypt(&compressed_public_graph, key)?,
		};

		Ok(PageBlob {
			page_id: self.page_id,
			content_hash: self.content_hash,
			payload: SchemaHandler::write_private_graph_chunk(&private_chunk)?,
		})
	}
}

impl Graph {
	/// Create a new, empty Graph
	pub fn new(connection_type: ConnectionType) -> Self {
		Self { connection_type, pages: PageMap::new() }
	}

	/// Getter for Pages in Graph
	pub fn pages(&self) -> &PageMap {
		&self.pages
	}

	/// Setter for Pages in Graph
	pub fn set_pages(&mut self, pages: PageMap) {
		self.pages = pages;
	}

	/// Get next available PageId for this graph
	pub fn get_next_available_page_id(&self) -> Option<PageId> {
		const MAX_PAGE_ID: PageId = 16; // todo: move this to config
		let existing_pages =
			self.pages.iter().map(|(page_id, _)| *page_id).collect::<HashSet<PageId>>();
		for pid in 0..MAX_PAGE_ID {
			if !existing_pages.contains(&pid) {
				return Some(pid)
			}
		}

		None
	}

	/// Remove all pages from this graph
	pub fn clear(&mut self) {
		self.pages.clear();
	}

	/// Import bundle of pages as a Public Graph
	pub fn import_public<E: EncryptionBehavior>(
		&mut self,
		ImportBundle::<E> { connection_type, pages, .. }: ImportBundle<E>,
	) -> Result<()> {
		if connection_type != self.connection_type {
			return Err(Error::msg("Incorrect connection type for graph import"))
		}

		let mut page_map = HashMap::<PageId, GraphPage>::new();
		for page in pages.iter() {
			match GraphPage::try_from(page) {
				Err(e) => return Err(e),
				Ok(p) => {
					page_map.insert(page.page_id, p);
				},
			};
		}

		self.set_pages(page_map);

		Ok(())
	}

	/// Import bundle of pages as a Private Graph
	pub fn import_private<E: EncryptionBehavior>(
		&mut self,
		ImportBundle::<E> { connection_type, pages, keys, .. }: ImportBundle<E>,
	) -> Result<()> {
		if connection_type != self.connection_type {
			return Err(Error::msg("Incorrect connection type for graph import"))
		}

		let mut page_map = PageMap::new();
		for page in pages.iter() {
			match GraphPage::try_from((page, &keys)) {
				Err(e) => return Err(e),
				Ok(p) => {
					page_map.insert(page.page_id, p);
				},
			};
		}

		self.set_pages(page_map);

		Ok(())
	}

	/// Import bundle of pages as a Private Graph, but without decrypting the encrypted portion
	/// Useful for importing a foreign user's graph for inspection of PRIds
	pub fn import_opaque<E: EncryptionBehavior>(
		&mut self,
		ImportBundle::<E> { connection_type, pages, .. }: ImportBundle<E>,
	) -> Result<()> {
		if connection_type.privacy_type() != PrivacyType::Private ||
			self.connection_type.privacy_type() != PrivacyType::Private
		{
			return Err(Error::msg("Invalid privacy type for opaque Private graph import"))
		}

		let mut page_map = PageMap::new();
		for page in pages.iter() {
			match DsnpUserPrivateGraphChunk::try_from(page.payload.as_slice()) {
				Err(e) => return Err(e.into()),
				Ok(chunk) => {
					page_map.insert(
						page.page_id,
						GraphPage {
							page_id: page.page_id,
							privacy_type: self.connection_type.privacy_type(),
							content_hash: page.content_hash,
							prids: chunk.prids,
							connections: Vec::new(),
						},
					);
				},
			};
		}

		self.set_pages(page_map);

		Ok(())
	}

	pub fn calculate_updates<E: EncryptionBehavior>(
		&self,
		updates: &Vec<UpdateEvent>,
		dsnp_user_id: &DsnpUserId,
		connection_keys: &Vec<DsnpKeys<E>>,
		encryption_key: (u64, &PublicKey<E>),
	) -> Result<ExportBundle> {
		let ids_to_remove: Vec<DsnpUserId> = updates
			.iter()
			.filter_map(|event| match event {
				UpdateEvent::Remove { dsnp_user_id, .. } => Some(*dsnp_user_id),
				_ => None,
			})
			.collect();

		let ids_to_add: Vec<DsnpUserId> = updates
			.iter()
			.filter_map(|event| match event {
				UpdateEvent::Add { dsnp_user_id, .. } => Some(*dsnp_user_id),
				_ => None,
			})
			.collect();

		// First calculate pages that have had connections removed. Later, we will
		// prefer to use these pages first to add new connections, so as to minimize
		// the number of pages to update.
		let pages_with_removals = self.find_connections(&ids_to_remove);

		let mut updated_pages: PageMap = self
			.pages
			.iter()
			.filter_map(|(page_id, page)| {
				if pages_with_removals.contains(page_id) {
					let mut updated_page = page.clone();
					updated_page.remove_connections(&ids_to_remove);
					return Some((*page_id, updated_page))
				}

				None
			})
			.collect();

		// Now try to add new connections into pages already being updated
		// Note: these pages have already been cloned, so we don't clone them again
		let mut add_iter = ids_to_add.iter();
		while let Some(_) = add_iter.clone().peekable().peek() {
			if let Some((_page_id, page)) =
				updated_pages.iter_mut().find(|(_, page)| !page.is_full(false))
			{
				let id_to_add = add_iter.next().unwrap();
				page.add_connection(id_to_add)?;
			}
		}

		// Now go through the remaining connections to be added and find space in
		// other existing pages to add them
		let mut current_page: Option<Box<GraphPage>> = None;
		while let Some(_) = add_iter.clone().peekable().peek() {
			if current_page.is_none() {
				let available_page = self.pages.iter().find(|(page_id, page)| {
					!updated_pages
						.iter()
						.map(|(id, _)| id)
						.collect::<Vec<&PageId>>()
						.contains(page_id) && !page.is_full(false)
				});

				current_page = match available_page {
					Some((_page_id, page)) => Some(Box::new(page.clone())),
					None => match self.get_next_available_page_id() {
						Some(next_page_id) => Some(Box::new(GraphPage::new(
							self.connection_type.privacy_type(),
							next_page_id,
						))),
						None => None,
					},
				}
			}

			match current_page {
				Some(ref mut page) => {
					page.add_connection(add_iter.next().unwrap())?;
					if page.is_full(false) {
						updated_pages.insert(page.page_id, *page.clone());
						current_page = None;
					}
				},
				None => return Err(Error::msg("Graph is full")), // todo: re-calculate updates with agressive fullness determination
			}

			if current_page.is_some() {
				let mut page = current_page.clone().unwrap();
				page.add_connection(add_iter.next().unwrap())?;
				if page.is_full(false) {
					updated_pages.insert(page.page_id, *page);
					current_page = None;
				}
			} else {
				return Err(Error::msg("Graph is full")) // todo: re-calculate updates with agressive fullness determination
			};
		}

		// If any pages now empty, add to the remove list
		let mut removed_pages: Vec<PageId> = Vec::new();
		updated_pages.retain(|page_id, page| {
			if page.is_empty() {
				removed_pages.push(*page_id);
				return false
			}
			true
		});

		if current_page.is_some() {
			let last_page = current_page.unwrap();
			updated_pages.insert(last_page.page_id, *last_page);
		}

		let updated_blobs: Result<Vec<PageBlob>> = match self.connection_type.privacy_type() {
			PrivacyType::Public =>
				updated_pages.iter().map(|(_, page)| page.to_public_blob()).collect(),
			PrivacyType::Private => updated_pages
				.iter_mut()
				.map(|(_, page)| page.to_private_blob(encryption_key, connection_keys))
				.collect(),
		};

		Ok(ExportBundle {
			dsnp_user_id: *dsnp_user_id,
			connection_type: self.connection_type,
			updated_pages: updated_blobs?,
			removed_pages,
		})
	}

	/// Create a new Page in the Graph, with the given PageId.
	///
	/// Error on duplicate PageId.
	/// If Some(Page) supplied, insert the given page.
	/// Otherwise, create a new empty page.
	pub fn create_page(
		&mut self,
		page_id: &PageId,
		page: Option<GraphPage>,
	) -> Result<&mut GraphPage, &str> {
		if let Some(_existing_page) = self.pages.get(page_id) {
			return Err("Attempt to create a new page for an existing page ID")
		}

		self.pages.insert(
			*page_id,
			match page {
				Some(page) => page,
				None => GraphPage::new(self.connection_type.privacy_type(), *page_id),
			},
		);
		Ok(self
			.pages
			.get_mut(page_id)
			.expect("Unable to retrieve graph page just inserted"))
	}

	/// Retrieve the page with the given PageId
	pub fn get_page(&self, page_id: &PageId) -> Option<&GraphPage> {
		self.pages.get(page_id)
	}

	/// Retrieve a mutable reference to the page with the given PageId
	pub fn get_page_mut(&mut self, page_id: &PageId) -> Option<&mut GraphPage> {
		self.pages.get_mut(page_id)
	}

	/// Boolean function to indicate if a connection is present in the graph
	pub fn has_connection(&self, dsnp_id: &DsnpUserId) -> bool {
		self.pages.iter().any(|(_, page)| page.contains(dsnp_id))
	}

	/// Return the PageId in which the given connection resides, if found.
	pub fn find_connection(&self, dsnp_id: &DsnpUserId) -> Option<PageId> {
		for (id, page) in self.pages.iter() {
			if page.contains(dsnp_id) {
				return Some(*id)
			}
		}

		None
	}

	/// Return all PageIds containing any of the connections in the list
	pub fn find_connections(&self, ids: &Vec<DsnpUserId>) -> Vec<PageId> {
		self.pages
			.iter()
			.filter_map(|(page_id, page)| match page.contains_any(ids) {
				true => Some(*page_id),
				false => None,
			})
			.collect()
	}

	/// Add a connection to the specified page.
	/// This is used internally by the Graph Update Manager or Import
	/// If the specified page does not exist, a new page will be created
	/// and the connection inserted into it.
	pub fn add_connection_to_page(
		&mut self,
		page_id: &PageId,
		connection_id: &DsnpUserId,
	) -> Result<()> {
		if self.find_connection(connection_id).is_some() {
			return Err(Error::msg("Add of duplicate connection in another page detected"))
		}

		if !self.pages.contains_key(page_id) {
			self.pages
				.insert(*page_id, GraphPage::new(self.connection_type.privacy_type(), *page_id));
		}
		let page = self.get_page_mut(page_id).expect("Unable to retrieve page");
		page.add_connection(connection_id)
	}

	/// Remove a connection from the graph.
	/// Returns Ok(Option<PageId>) containing the PageId of the page
	/// the connection was removed from, or Ok(None) if the connection
	/// was not found.
	pub fn remove_connection(&mut self, connection_id: &DsnpUserId) -> Result<Option<PageId>> {
		if let Some(page_id) = self.find_connection(connection_id) {
			return match self.get_page_mut(&page_id) {
				Some(page) => match page.remove_connection(connection_id) {
					Ok(()) => Ok(Some(page_id)),
					Err(e) => Err(e),
				},
				None => Err(Error::msg("Unable to retrieve page")),
			}
		}

		// Return Ok if no-op/connection not found
		Ok(None)
	}
}

/// Macro to get an iterator to all connections across all GraphPages
/// within a Graph.
#[macro_export]
macro_rules! iter_graph_connections {
	( $x:expr ) => {{
		$x.pages()
			.values()
			.flat_map(|p| p.connections().iter().cloned())
			.collect::<Vec<DsnpGraphEdge>>()
			.iter()
	}};
}
