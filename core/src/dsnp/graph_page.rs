#![allow(dead_code)] // todo: remove
use crate::{
	dsnp::{api_types::*, dsnp_types::*},
	util::time::time_in_ksecs,
};
use anyhow::{Error, Result};
use std::collections::HashMap;

use super::{
	compression::{CompressionBehavior, DeflateCompression},
	encryption::EncryptionBehavior,
	schema::SchemaHandler,
};

/// Graph page structure
#[derive(Debug, Clone, PartialEq)]
pub struct GraphPage {
	content_hash: u32,
	/// List of PRIds
	prids: Vec<DsnpPrid>,
	/// List of connections
	connections: Vec<DsnpGraphEdge>,
	/// Days since Unix epoch when PRIds were last refreshed for this page
	last_updated: u32,
}

/// Conversion for Public Graph
impl TryFrom<&PageBlob> for GraphPage {
	type Error = Error;
	fn try_from(PageBlob { content_hash, payload, .. }: &PageBlob) -> Result<Self> {
		let DsnpUserPublicGraphChunk { compressed_public_graph } =
			DsnpUserPublicGraphChunk::try_from(payload.as_slice())?;
		let uncompressed_chunk = DeflateCompression::decompress(&compressed_public_graph)?;
		Ok(Self {
			content_hash: *content_hash,
			prids: Vec::new(),
			connections: SchemaHandler::read_inner_graph(&uncompressed_chunk)?,
			last_updated: 0,
		})
	}
}

/// Conversion for Private Graph
impl<E: EncryptionBehavior> TryFrom<(&PageBlob, &Vec<KeyPair<E>>)> for GraphPage {
	type Error = Error;
	fn try_from(
		(PageBlob { content_hash, payload, .. }, keys): (&PageBlob, &Vec<KeyPair<E>>),
	) -> Result<Self> {
		let DsnpUserPrivateGraphChunk {
			key_id,
			encrypted_compressed_private_graph,
			prids,
			last_updated,
		} = DsnpUserPrivateGraphChunk::try_from(payload.as_slice())?;
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

				Ok(GraphPage { content_hash: *content_hash, prids, connections, last_updated })
			},
		}
	}
}

/// Graph structure to hold pages of connections of a single type
#[derive(Debug, Clone)]
pub struct Graph {
	pages: HashMap<PageId, GraphPage>,
}

/// Structure to hold all of a User's Graphs, mapped by ConnectionType
#[derive(Debug, Clone)]
pub struct UserGraph {
	graphs: HashMap<ConnectionType, Graph>,
}

impl UserGraph {
	/// Create a new, empty UserGraph
	pub fn new() -> Self {
		Self {
			graphs: HashMap::<ConnectionType, Graph>::from([
				(ConnectionType::Follow(PrivacyType::Public), Graph::new()),
				(ConnectionType::Follow(PrivacyType::Private), Graph::new()),
				(ConnectionType::Friendship(PrivacyType::Public), Graph::new()),
				(ConnectionType::Friendship(PrivacyType::Private), Graph::new()),
			]),
		}
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
}

impl GraphPage {
	/// Create a new, empty page
	pub fn new() -> Self {
		Self {
			content_hash: 0,
			prids: Vec::<DsnpPrid>::new(),
			connections: Vec::<DsnpGraphEdge>::new(),
			last_updated: 0,
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

	/// Tester to check if the page contains a connection to a particular DsnpUserId
	pub fn contains(&self, connection_id: &DsnpUserId) -> bool {
		self.connections.iter().any(|c| c.user_id == *connection_id)
	}

	/// Function to test if the page is empty
	pub fn is_empty(&self) -> bool {
		self.connections.is_empty()
	}

	/// Add a connection to the page. Fail if the connection is already present.
	pub fn add_connection(&mut self, connection_id: &DsnpUserId) -> Result<(), &str> {
		if self.contains(connection_id) {
			return Err("Add of duplicate connection detected")
		}

		self.connections
			.push(DsnpGraphEdge { user_id: *connection_id, since: time_in_ksecs() });
		Ok(())
	}

	/// Remove a connection from the page. Error if connection not found in page.
	pub fn remove_connection(&mut self, connection_id: &DsnpUserId) -> Result<(), &str> {
		if !self.contains(connection_id) {
			return Err("Connection not found in page")
		}

		self.connections.retain(|c| c.user_id != *connection_id);
		Ok(())
	}
}

impl Graph {
	/// Create a new, empty Graph
	pub fn new() -> Self {
		Self { pages: HashMap::<PageId, GraphPage>::new() }
	}

	/// Getter for Pages in Graph
	pub fn pages(&self) -> &HashMap<PageId, GraphPage> {
		&self.pages
	}

	/// Setter for Pages in Graph
	pub fn set_pages(&mut self, pages: HashMap<PageId, GraphPage>) {
		self.pages = pages;
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
		if connection_type.privacy_type() != PrivacyType::Public {
			return Err(Error::msg("Invalid privacy type for Public graph import"))
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
		if connection_type.privacy_type() != PrivacyType::Private {
			return Err(Error::msg("Invalid privacy type for Private graph import"))
		}

		let mut page_map = HashMap::<PageId, GraphPage>::new();
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
		if connection_type.privacy_type() != PrivacyType::Private {
			return Err(Error::msg("Invalid privacy type for opaque Private graph import"))
		}

		let mut page_map = HashMap::<PageId, GraphPage>::new();
		for page in pages.iter() {
			match DsnpUserPrivateGraphChunk::try_from(page.payload.as_slice()) {
				Err(e) => return Err(e.into()),
				Ok(chunk) => {
					page_map.insert(
						page.page_id,
						GraphPage {
							content_hash: page.content_hash,
							prids: chunk.prids,
							connections: Vec::new(),
							last_updated: chunk.last_updated,
						},
					);
				},
			};
		}

		self.set_pages(page_map);

		Ok(())
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
				None => GraphPage::new(),
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

	/// Return the PageId in which the given connection resides, if found.
	pub fn find_connection(&self, dsnp_id: &DsnpUserId) -> Option<PageId> {
		for (id, page) in self.pages.iter() {
			if page.contains(dsnp_id) {
				return Some(*id)
			}
		}

		None
	}

	/// Add a connection to the specified page.
	/// This is used internally by the Graph Update Manager or Import
	/// If the specified page does not exist, a new page will be created
	/// and the connection inserted into it.
	pub fn add_connection_to_page<'a>(
		&'a mut self,
		page_id: &PageId,
		connection_id: &DsnpUserId,
	) -> Result<(), &str> {
		if self.find_connection(connection_id).is_some() {
			return Err("Add of duplicate connection in another page detected")
		}

		if !self.pages.contains_key(page_id) {
			self.pages.insert(*page_id, GraphPage::new());
		}
		let page = self.get_page_mut(page_id).expect("Unable to retrieve page");
		page.add_connection(connection_id)
	}

	/// Remove a connection from the graph.
	/// Returns Ok(Option<PageId>) containing the PageId of the page
	/// the connection was removed from, or Ok(None) if the connection
	/// was not found.
	pub fn remove_connection(
		&mut self,
		connection_id: &DsnpUserId,
	) -> Result<Option<PageId>, &str> {
		if let Some(page_id) = self.find_connection(connection_id) {
			return match self.get_page_mut(&page_id) {
				Some(page) => match page.remove_connection(connection_id) {
					Ok(()) => Ok(Some(page_id)),
					Err(e) => Err(e),
				},
				None => Err("Unable to retrieve page"),
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
