use crate::{
	dsnp::{api_types::*, dsnp_types::*},
	graph::updates::UpdateEvent,
};
use anyhow::{Error, Result};
use dsnp_graph_config::{Environment, SchemaId};
use std::collections::{HashMap, HashSet};

use super::page::GraphPage;
use crate::dsnp::encryption::EncryptionBehavior;

pub type PageMap = HashMap<PageId, GraphPage>;
pub const MAX_PAGE_ID: PageId = 16; // todo: move this to config

/// Graph structure to hold pages of connections of a single type
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Graph {
	environment: Environment,
	schema_id: SchemaId,
	pages: PageMap,
}

impl Graph {
	/// Create a new, empty Graph
	pub fn new(environment: Environment, schema_id: SchemaId) -> Self {
		Self { environment, schema_id, pages: PageMap::new() }
	}

	/// Get total number of connections in graph
	pub fn len(&self) -> usize {
		self.pages.values().flat_map(|p| p.connections()).count()
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
		let existing_pages = self.pages.keys().cloned().collect::<HashSet<PageId>>();
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

	pub fn get_connection_type(&self) -> ConnectionType {
		self.environment
			.get_config()
			.get_connection_type_from_schema_id(self.schema_id)
			.expect("Connection type should exist!")
	}

	/// Import bundle of pages as a Public Graph
	pub fn import_public(
		&mut self,
		connection_type: ConnectionType,
		pages: Vec<PageData>,
	) -> Result<()> {
		if connection_type != self.get_connection_type() {
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
	pub fn import_private(
		&mut self,
		connection_type: ConnectionType,
		pages: &[PageData],
		keys: Vec<ResolvedKeyPair>,
	) -> Result<()> {
		if connection_type != self.get_connection_type() {
			return Err(Error::msg("Incorrect connection type for graph import"))
		}

		let dsnp_config = self
			.environment
			.get_config()
			.get_dsnp_config_from_schema_id(self.schema_id)
			.expect("DsnpConfig should exist!")
			.1;
		let mut page_map = PageMap::new();
		for page in pages.iter() {
			match GraphPage::try_from((page, &dsnp_config, &keys)) {
				Err(e) => return Err(e.into()),
				Ok(p) => {
					page_map.insert(page.page_id, p);
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
		connection_keys: &Vec<DsnpKeys>,
		encryption_key: (u64, &PublicKey<E>),
	) -> Result<Vec<Update>> {
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
				let id_to_add = add_iter.next().unwrap(); // safe to unwrap because we peeked above
				page.add_connection(id_to_add)?;
			} else {
				break
			}
		}

		// Now go through the remaining connections to be added and find space in
		// other existing pages to add them
		let mut current_page: Option<Box<GraphPage>> = None;
		while let Some(_) = add_iter.clone().peekable().peek() {
			if current_page.is_none() {
				let available_page = self.pages.iter().find(|(page_id, page)| {
					!updated_pages.contains_key(page_id) && !page.is_full(false)
				});

				current_page = match available_page {
					Some((_page_id, page)) => Some(Box::new(page.clone())),
					None => match self.get_next_available_page_id() {
						Some(next_page_id) => Some(Box::new(GraphPage::new(
							self.get_connection_type().privacy_type(),
							next_page_id,
						))),
						None => None,
					},
				}
			}

			match current_page {
				Some(ref mut page) => {
					page.add_connection(add_iter.next().unwrap())?; // safe to unwrap because we peeked above
					if page.is_full(false) {
						updated_pages.insert(page.page_id(), *page.clone());
						current_page = None;
					}
				},
				None => return Err(Error::msg("Graph is full")), // todo: re-calculate updates with agressive fullness determination
			}
		}

		// If any pages now empty, add to the remove list
		let mut removed_pages: Vec<PageData> = Vec::new();
		updated_pages.retain(|_, page| {
			if page.is_empty() {
				removed_pages.push(page.to_removed_page_data());
				return false
			}
			true
		});

		if let Some(last_page) = current_page {
			updated_pages.insert(last_page.page_id(), *last_page);
		}

		let updated_blobs: Result<Vec<PageData>> = match self.get_connection_type().privacy_type() {
			PrivacyType::Public =>
				updated_pages.values().map(|page| page.to_public_page_data()).collect(),
			PrivacyType::Private => updated_pages
				.iter_mut()
				.map(|(_, page)| page.to_private_page_data::<E>(encryption_key, connection_keys))
				.collect(),
		};

		let mut updates: Vec<Update> = updated_blobs?
			.iter()
			.map(|page_data| Update::PersistPage {
				owner_dsnp_user_id: *dsnp_user_id,
				schema_id: self.schema_id,
				page_id: page_data.page_id,
				prev_hash: page_data.content_hash,
				payload: page_data.content.clone(),
			})
			.collect();

		updates.extend(removed_pages.iter().map(|p| Update::DeletePage {
			owner_dsnp_user_id: *dsnp_user_id,
			schema_id: self.schema_id,
			page_id: p.page_id,
			prev_hash: p.content_hash,
		}));

		Ok(updates)
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
				None => GraphPage::new(self.get_connection_type().privacy_type(), *page_id),
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
			self.pages.insert(
				*page_id,
				GraphPage::new(self.get_connection_type().privacy_type(), *page_id),
			);
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

#[cfg(test)]
mod test {
	use super::*;
	use crate::{
		dsnp::encryption::SealBox,
		tests::helpers::{
			avro_public_payload, create_test_graph, create_test_ids_and_page, PageDataBuilder,
			INNER_TEST_DATA,
		},
	};
	use dryoc::keypair::StackKeyPair;
	use ntest::*;
	#[allow(unused_imports)]
	use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
	use std::collections::HashMap;

	#[test]
	fn new_graph_is_empty() {
		let env = Environment::Mainnet;
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(ConnectionType::Follow(PrivacyType::Private))
			.expect("should exist");
		let graph = Graph::new(env, schema_id);
		assert_eq!(graph.pages().is_empty(), true);
	}

	#[test]
	fn graph_len_reports_number_of_connections() {
		let graph = create_test_graph();
		assert_eq!(graph.len(), 25);
	}

	#[test]
	fn page_setter_sets_pages() {
		let env = Environment::Mainnet;
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(ConnectionType::Follow(PrivacyType::Private))
			.expect("should exist");
		let mut pages = HashMap::<PageId, GraphPage>::new();
		for i in 0..=1 {
			let (_, p) = create_test_ids_and_page();
			pages.insert(i, p);
		}
		let mut graph = Graph::new(env, schema_id);
		graph.set_pages(pages.clone());
		assert_eq!(pages.len(), graph.pages().len());
		for i in 0..pages.len() as u16 {
			assert_eq!(pages.get(&i), graph.pages().get(&i));
		}
	}

	#[test]
	fn get_next_available_page_id_returns_none_for_full_graph() {
		let environment = Environment::Mainnet;
		const CONN_TYPE: ConnectionType = ConnectionType::Follow(PrivacyType::Public);
		const PRIV_TYPE: PrivacyType = CONN_TYPE.privacy_type();
		let schema_id = environment
			.get_config()
			.get_schema_id_from_connection_type(CONN_TYPE)
			.expect("should exist");
		let pages: HashMap<PageId, GraphPage> = (0..MAX_PAGE_ID)
			.map(|page_id: PageId| (page_id, GraphPage::new(PRIV_TYPE, page_id)))
			.collect();
		let graph = Graph {
			environment,
			schema_id, // doesn't matter which type
			pages,
		};

		assert_eq!(graph.get_next_available_page_id(), None);
	}

	#[test]
	fn get_next_available_page_id_returns_correct_value() {
		let environment = Environment::Mainnet;
		const CONN_TYPE: ConnectionType = ConnectionType::Follow(PrivacyType::Public);
		const PRIV_TYPE: PrivacyType = CONN_TYPE.privacy_type();
		let schema_id = environment
			.get_config()
			.get_schema_id_from_connection_type(CONN_TYPE)
			.expect("should exist");
		let mut pages: HashMap<PageId, GraphPage> = (0..MAX_PAGE_ID)
			.map(|page_id: PageId| (page_id, GraphPage::new(PRIV_TYPE, page_id)))
			.collect();
		pages.remove(&8);
		let graph = Graph {
			environment,
			schema_id, // doesn't matter which type
			pages,
		};

		assert_eq!(graph.get_next_available_page_id(), Some(8));
	}

	#[test]
	fn clear_removes_all_pages() {
		let mut graph = create_test_graph();
		assert_eq!(graph.pages.len() > 0, true);
		graph.clear();
		assert_eq!(graph.pages.len(), 0);
	}

	#[test]
	fn import_public_gets_correct_data() {
		let environment = Environment::Mainnet;
		let connection_type = ConnectionType::Follow(PrivacyType::Public);
		let schema_id = environment
			.get_config()
			.get_schema_id_from_connection_type(connection_type)
			.expect("should exist");
		let mut graph = Graph::new(environment, schema_id);
		let blob = PageData { content_hash: 0, page_id: 0, content: avro_public_payload() };
		let pages = vec![blob];

		let _ = graph.import_public(connection_type, pages);
		assert_eq!(graph.pages.len(), 1);
		let orig_connections: HashSet<DsnpUserId> =
			INNER_TEST_DATA.iter().map(|edge| edge.user_id).collect();
		let imported_connections: HashSet<DsnpUserId> =
			iter_graph_connections!(graph).map(|edge| edge.user_id).collect();
		assert_eq!(orig_connections, imported_connections);
	}

	#[test]
	fn import_private_gets_correct_data() {
		let connection_type = ConnectionType::Follow(PrivacyType::Private);
		let environment = Environment::Mainnet;
		let schema_id = environment
			.get_config()
			.get_schema_id_from_connection_type(connection_type)
			.expect("should exist");
		let mut graph = Graph::new(environment, schema_id);
		let resolved_key = ResolvedKeyPair { key_pair: StackKeyPair::gen(), key_id: 1 };
		let orig_connections: HashSet<DsnpUserId> =
			INNER_TEST_DATA.iter().map(|edge| edge.user_id).collect();
		let pages = PageDataBuilder::new(connection_type)
			.with_encryption_key(resolved_key.clone())
			.with_page(0, &orig_connections.iter().cloned().collect::<Vec<_>>(), &vec![])
			.build();

		let res = graph.import_private(connection_type, &pages, vec![resolved_key]);

		assert!(res.is_ok());
		assert_eq!(graph.pages.len(), 1);
		let imported_connections: HashSet<DsnpUserId> =
			iter_graph_connections!(graph).map(|edge| edge.user_id).collect();
		assert_eq!(orig_connections, imported_connections);
	}

	#[test]
	fn create_page_with_existing_pageid_fails() {
		let mut graph = create_test_graph();

		assert_eq!(graph.create_page(&0, None).is_err(), true);
	}

	#[test]
	fn create_page_succeeds() {
		let environment = Environment::Mainnet;
		let schema_id = environment
			.get_config()
			.get_schema_id_from_connection_type(ConnectionType::Follow(PrivacyType::Private))
			.expect("should exist");
		let (_, page) = create_test_ids_and_page();
		let mut graph = Graph::new(environment, schema_id);

		assert_eq!(graph.create_page(&0, Some(page.clone())).is_ok(), true);
		assert_eq!(page, *graph.get_page(&0).unwrap());
	}

	#[test]
	fn has_connection_returns_false_for_missing_connection() {
		let graph = create_test_graph();

		assert_eq!(graph.has_connection(&99), false);
	}

	#[test]
	fn has_connection_returns_true_for_present_connection() {
		let graph = create_test_graph();

		assert_eq!(graph.has_connection(&1), true);
	}

	#[test]
	fn find_connection_returns_none_for_nonexistent_connection() {
		let graph = create_test_graph();

		assert_eq!(graph.find_connection(&99), None);
	}

	#[test]
	fn find_connections_returns_pageid_of_existing_connection() {
		let graph = create_test_graph();

		assert_eq!(graph.find_connection(&1), Some(0));
	}

	#[test]
	fn find_connections_returns_vec_of_pageids() {
		let graph = create_test_graph();

		let mut v = graph.find_connections(&vec![1, 5, 24]);
		v.sort();
		assert_eq!(v, vec![0, 1, 4]);
	}

	#[test]
	fn add_connection_duplicate_connection_errors() {
		let mut graph = create_test_graph();

		assert_eq!(graph.add_connection_to_page(&4, &0).is_err(), true);
	}

	#[test]
	fn add_connection_to_nonexistent_page_adds_new_page() {
		let mut graph = create_test_graph();
		let page_to_add: PageId = 99;

		assert_eq!(graph.pages().contains_key(&page_to_add), false);
		let _ = graph.add_connection_to_page(&page_to_add, &12345);
		assert_eq!(graph.pages().contains_key(&page_to_add), true);
	}

	#[test]
	fn add_connection_succeeds() {
		let mut graph = create_test_graph();

		let _ = graph.add_connection_to_page(&4, &99);
		assert_eq!(graph.find_connection(&99), Some(4));
	}

	#[test]
	fn remove_connection_returns_none_for_not_found() {
		let mut graph = create_test_graph();

		let result = graph.remove_connection(&99);
		assert_eq!(result.unwrap().is_none(), true);
	}

	#[test]
	fn remove_connection_returns_pageid_of_removed_connection() {
		let mut graph = create_test_graph();

		let result = graph.remove_connection(&5);
		assert_eq!(result.unwrap(), Some(1));
	}

	#[test]
	fn graph_iterator_should_iterate_over_all_connections() {
		let graph = create_test_graph();
		let mut test_connections: Vec<DsnpUserId> = (0..25).map(|i| i as DsnpUserId).collect();
		test_connections.sort();

		let mut graph_connections: Vec<DsnpUserId> =
			iter_graph_connections!(graph).map(|edge| edge.user_id).collect();
		graph_connections.sort();
		assert_eq!(test_connections, graph_connections);
	}

	#[test]
	#[timeout(5000)] // found an infinite loop bug, so let's make sure this terminates successfully
	fn calculate_public_updates_succeeds() {
		let graph = create_test_graph();
		let updates = vec![UpdateEvent::create_add(1, graph.schema_id)].to_vec();
		let _ = graph.calculate_updates::<SealBox>(
			&updates,
			&0,
			&Vec::<DsnpKeys>::new(),
			(0, &<SealBox as EncryptionBehavior>::EncryptionInput::new()),
		);
	}
}
