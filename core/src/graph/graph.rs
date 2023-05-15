#![allow(dead_code)]
use crate::{
	dsnp::{api_types::*, dsnp_configs::DsnpVersionConfig, dsnp_types::*},
	graph::{
		key_manager::UserKeyManagerBase,
		page::{PrivatePageDataProvider, PublicPageDataProvider, RemovedPageDataProvider},
		page_capacities::PAGE_CAPACITY_MAP,
		updates::UpdateEvent,
	},
	util::{
		time::duration_days_since,
		transactional_hashmap::{Transactional, TransactionalHashMap},
	},
};
use anyhow::{Error, Result};
use dsnp_graph_config::{Environment, SchemaId};
use std::{
	cell::RefCell,
	collections::{BTreeMap, HashMap, HashSet},
	ops::Deref,
	rc::Rc,
};

use super::page::GraphPage;

pub type PageMap = TransactionalHashMap<PageId, GraphPage>;

/// Graph structure to hold pages of connections of a single type
#[derive(Debug, Clone)]
pub struct Graph {
	environment: Environment,
	user_id: DsnpUserId,
	schema_id: SchemaId,
	pages: PageMap,
	user_key_manager: Rc<RefCell<dyn UserKeyManagerBase>>,
}

impl PartialEq for Graph {
	fn eq(&self, other: &Self) -> bool {
		self.environment == other.environment &&
			self.user_id == other.user_id &&
			self.schema_id == other.schema_id &&
			self.pages.eq(&other.pages)
	}
}

impl Transactional for Graph {
	fn commit(&mut self) {
		let page_ids: Vec<_> = self.pages.inner().keys().copied().collect();
		for pid in page_ids {
			if let Some(g) = self.pages.get_mut(&pid) {
				g.commit();
			}
		}
		self.pages.commit();
	}

	fn rollback(&mut self) {
		self.pages.rollback();
		let page_ids: Vec<_> = self.pages.inner().keys().copied().collect();
		for pid in page_ids {
			if let Some(g) = self.pages.get_mut(&pid) {
				g.rollback();
			}
		}
	}
}

impl Graph {
	/// Create a new, empty Graph
	pub fn new<E>(
		environment: Environment,
		user_id: DsnpUserId,
		schema_id: SchemaId,
		user_key_manager: Rc<RefCell<E>>,
	) -> Self
	where
		E: UserKeyManagerBase + 'static,
	{
		Self { environment, user_id, schema_id, pages: PageMap::new(), user_key_manager }
	}

	/// Get total number of connections in graph
	pub fn len(&self) -> usize {
		self.pages.inner().values().flat_map(|p| p.connections()).count()
	}

	/// Getter for Pages in Graph
	pub fn pages(&self) -> &PageMap {
		&self.pages
	}

	/// Setter for Pages in Graph
	#[cfg(test)]
	pub fn set_pages(&mut self, pages: PageMap) {
		self.pages = pages;
	}

	/// Get next available PageId for this graph
	pub fn get_next_available_page_id(&self) -> Option<PageId> {
		let existing_pages = self.pages.inner().keys().cloned().collect::<HashSet<PageId>>();
		for pid in 0..=(self.environment.get_config().max_page_id as PageId) {
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

	pub fn get_schema_id(&self) -> SchemaId {
		self.schema_id
	}

	/// Import bundle of pages as a Public Graph
	pub fn import_public(
		&mut self,
		connection_type: ConnectionType,
		pages: &Vec<PageData>,
	) -> Result<()> {
		if connection_type != self.get_connection_type() {
			return Err(Error::msg("Incorrect connection type for graph import"))
		}
		let max_page_id = self.environment.get_config().max_page_id;
		let mut page_map = HashMap::new();
		for page in pages.iter() {
			if page.page_id > max_page_id as PageId {
				return Err(Error::msg(format!(
					"Imported page has an invalid page Id {}",
					page.page_id
				)))
			}
			match GraphPage::try_from(page) {
				Err(e) => return Err(e),
				Ok(p) => {
					page_map.insert(page.page_id, p);
				},
			};
		}

		self.pages.clear();
		for (page_id, page) in page_map {
			self.pages.insert(page_id, page);
		}

		Ok(())
	}

	/// Import bundle of pages as a Private Graph
	pub fn import_private(
		&mut self,
		dsnp_version_config: &DsnpVersionConfig,
		connection_type: ConnectionType,
		pages: &[PageData],
	) -> Result<()> {
		if connection_type != self.get_connection_type() {
			return Err(Error::msg("Incorrect connection type for graph import"))
		}

		let max_page_id = self.environment.get_config().max_page_id;
		let keys = self.user_key_manager.deref().borrow().get_all_resolved_keys();
		let mut page_map = HashMap::new();
		for page in pages.iter() {
			if page.page_id > max_page_id as PageId {
				return Err(Error::msg(format!(
					"Imported page has an invalid page Id {}",
					page.page_id
				)))
			}
			match GraphPage::try_from((page, dsnp_version_config, &keys)) {
				Err(e) => return Err(e.into()),
				Ok(p) => {
					page_map.insert(page.page_id, p);
				},
			};
		}

		self.pages.clear();
		for (page_id, page) in page_map {
			self.pages.insert(page_id, page);
		}

		Ok(())
	}

	pub fn calculate_updates(
		&self,
		dsnp_version_config: &DsnpVersionConfig,
		updates: &Vec<UpdateEvent>,
	) -> Result<Vec<Update>> {
		let encryption_key = match self.get_connection_type().privacy_type() {
			PrivacyType::Public => None,
			PrivacyType::Private =>
				self.user_key_manager.borrow().get_resolved_active_key(self.user_id),
		};

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

		// using tree-map to keep the order of pages consistent in update process
		let mut updated_pages: BTreeMap<PageId, GraphPage> = self
			.pages
			.inner()
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
		let mut add_iter = ids_to_add.iter().peekable();
		for aggressive in vec![false, true] {
			for page in updated_pages.values_mut() {
				while let Some(id_to_add) = add_iter.peek() {
					if let Ok(_) = self.try_add_connection_to_page(
						page,
						id_to_add,
						aggressive,
						&encryption_key,
						Some(dsnp_version_config),
					) {
						let _ = add_iter.next(); // TODO: prefer advance_by(1) once that stabilizes
						continue
					} else {
						break
					}
				}
			}
		}

		// Now go through the remaining connections to be added and see if we can
		// add them to other existing pages that are non-full. Here we prefer to only
		// aggressively scan pages for fullness, because we want to minimize the number
		// of additional pages to be updated.
		let updated_keys: HashSet<PageId> = updated_pages.keys().cloned().collect();
		'existing_page_loop: for (_, page) in
			self.pages.inner().iter().filter(|(page_id, _)| !updated_keys.contains(page_id))
		{
			let mut current_page = page.clone();
			let mut page_modified = false;
			while let Some(id_to_add) = add_iter.peek() {
				if let Ok(_) = self.try_add_connection_to_page(
					&mut current_page,
					id_to_add,
					true,
					&encryption_key,
					Some(dsnp_version_config),
				) {
					page_modified = true;
					let _ = add_iter.next(); // TODO: prefer advance_by(1) once stabilized
						 // If no more connections to add, we're done. Otherwise we'll continue
						 // adding connections to the current page until either it's full, or
						 // we have no more connections to add.
					if let None = add_iter.peek() {
						updated_pages.insert(current_page.page_id(), current_page);
						break 'existing_page_loop
					}
				}
				// If we couldn't add a connection to the current page, it's full. Add this page
				// to the updated pages list and move on to the next page.
				else {
					if page_modified {
						updated_pages.insert(current_page.page_id(), current_page.clone());
					}
					continue 'existing_page_loop
				}
			}
		}

		// At this point, all existing pages are aggressively full. Add new pages
		// as needed to accommodate any remaining connections to be added, filling aggressively.
		let mut new_page: Option<Box<GraphPage>> = None;
		while let Some(id_to_add) = add_iter.peek() {
			if new_page.is_none() {
				if let Some(next_page_id) = self.get_next_available_page_id() {
					new_page = Some(Box::new(GraphPage::new(
						self.get_connection_type().privacy_type(),
						next_page_id,
					)));
				} else {
					return Err(Error::msg("Graph is full; no new pages available"))
				}
			}

			if let Some(page) = new_page.as_mut() {
				if self
					.try_add_connection_to_page(
						&mut **page,
						id_to_add,
						true,
						&encryption_key,
						Some(dsnp_version_config),
					)
					.is_ok()
				{
					let _ = add_iter.next(); // TODO: prefer advance_by(1) once stabilized
					if add_iter.peek().is_none() {
						updated_pages.insert(page.page_id(), *page.clone());
						new_page = None;
					}
				} else {
					new_page = None;
					continue
				}
			}
		}

		if let Some(last_added_page) = new_page {
			updated_pages.insert(last_added_page.page_id(), *last_added_page);
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

		let updated_blobs: Result<Vec<PageData>> = match self.get_connection_type() {
			ConnectionType::Follow(PrivacyType::Public) |
			ConnectionType::Friendship(PrivacyType::Public) =>
				updated_pages.values().map(|page| page.to_public_page_data()).collect(),
			ConnectionType::Follow(PrivacyType::Private) => {
				let encryption_key =
					encryption_key.ok_or(Error::msg("No resolved active key found!"))?;
				updated_pages
					.iter_mut()
					.map(|(_, page)| {
						page.clear_prids();
						page.to_private_page_data(dsnp_version_config, &encryption_key)
					})
					.collect()
			},
			ConnectionType::Friendship(PrivacyType::Private) => {
				let encryption_key =
					encryption_key.ok_or(Error::msg("No resolved active key found!"))?;
				updated_pages
					.iter_mut()
					.map(|(_, page)| {
						let mut updated_page = page.clone();
						self.apply_prids(&mut updated_page, &ids_to_add, &encryption_key)?;
						updated_page.to_private_page_data(dsnp_version_config, &encryption_key)
					})
					.collect()
			},
		};

		let updates: Vec<Update> = updated_blobs?
			.into_iter()
			.chain(removed_pages.into_iter())
			.map(|page_data| Update::from((page_data, self.user_id, self.schema_id)))
			.collect();
		Ok(updates)
	}

	/// recalculates and export pages, can be used to rotate keys or refresh PRID or remove empty
	/// pages
	pub fn force_recalculate(
		&self,
		dsnp_version_config: &DsnpVersionConfig,
	) -> Result<Vec<Update>> {
		// get latest encryption key
		let encryption_key = match self.get_connection_type().privacy_type() {
			PrivacyType::Public => None,
			PrivacyType::Private =>
				self.user_key_manager.borrow().get_resolved_active_key(self.user_id),
		};

		let mut updates = vec![];

		// calculate all pages
		for (_, page) in self.pages.inner() {
			let page_data_result = match page.is_empty() {
				true => Ok(page.to_removed_page_data()),
				false => match self.get_connection_type() {
					ConnectionType::Follow(PrivacyType::Public) |
					ConnectionType::Friendship(PrivacyType::Public) => page.to_public_page_data(),
					ConnectionType::Follow(PrivacyType::Private) => {
						let encryption_key = encryption_key
							.clone()
							.ok_or(Error::msg("No resolved active key found!"))?;
						let mut updated_page = page.clone();
						updated_page.clear_prids();
						updated_page.to_private_page_data(dsnp_version_config, &encryption_key)
					},
					ConnectionType::Friendship(PrivacyType::Private) => {
						let encryption_key = encryption_key
							.clone()
							.ok_or(Error::msg("No resolved active key found!"))?;
						let mut updated_page = page.clone();
						self.apply_prids(&mut updated_page, &vec![], &encryption_key)?;
						updated_page.to_private_page_data(dsnp_version_config, &encryption_key)
					},
				},
			};
			updates.push(page_data_result?);
		}

		// map to Update type
		let mapped = updates
			.into_iter()
			.map(|page_data| Update::from((page_data, self.user_id, self.schema_id)))
			.collect();
		Ok(mapped)
	}

	/// Create a new Page in the Graph, with the given PageId.
	///
	/// Error on duplicate PageId.
	/// If Some(Page) supplied, insert the given page.
	/// Otherwise, create a new empty page.
	#[cfg(test)]
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
		self.pages.inner().iter().any(|(_, page)| page.contains(dsnp_id))
	}

	/// Return the PageId in which the given connection resides, if found.
	pub fn find_connection(&self, dsnp_id: &DsnpUserId) -> Option<PageId> {
		for (id, page) in self.pages.inner().iter() {
			if page.contains(dsnp_id) {
				return Some(*id)
			}
		}

		None
	}

	/// Return all PageIds containing any of the connections in the list
	pub fn find_connections(&self, ids: &Vec<DsnpUserId>) -> Vec<PageId> {
		self.pages
			.inner()
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

		if !self.pages.inner().contains_key(page_id) {
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

	/// returns one sided friendship connections
	pub fn get_one_sided_friendships(&self) -> Result<Vec<DsnpGraphEdge>> {
		if self.get_connection_type() != ConnectionType::Friendship(PrivacyType::Private) {
			return Err(Error::msg(
				"Calling get_all_one_sided_friendships in non private friendship graph!",
			))
		}

		let mut result = vec![];
		for c in self.pages.inner().values().flat_map(|g| g.connections()) {
			if !self.user_key_manager.borrow().verify_connection(c.user_id)? {
				result.push(*c)
			}
		}
		Ok(result)
	}

	/// verifies prids for friendship from other party and calculates for own side
	fn apply_prids(
		&self,
		updated_page: &mut GraphPage,
		ids_to_add: &Vec<DsnpUserId>,
		encryption_key: &ResolvedKeyPair,
	) -> Result<()> {
		if self.get_connection_type() != ConnectionType::Friendship(PrivacyType::Private) {
			return Err(Error::msg("Calling apply_prids in non private friendship graph!"))
		}

		// verify connection existence based on prid
		let max_allowed_stale_days =
			self.environment.get_config().sdk_max_stale_friendship_days as u64;
		for c in updated_page
			.connections()
			.clone()
			.iter()
			.filter(|c| !ids_to_add.contains(&c.user_id))
		{
			if duration_days_since(c.since) > max_allowed_stale_days &&
				!self.user_key_manager.borrow().verify_connection(c.user_id)?
			{
				// connection is removed from the other side
				updated_page.remove_connection(&c.user_id)?;
			}
		}

		// calculating updated prids
		let prid_result: Result<Vec<_>> = updated_page
			.connections()
			.iter()
			.map(|c| {
				self.user_key_manager.borrow().calculate_prid(
					self.user_id,
					c.user_id,
					encryption_key.key_pair.clone().into(),
				)
			})
			.collect();
		updated_page.set_prids(prid_result?)
	}

	/// Determine if page is full
	///  aggressive:false -> use a simple heuristic based on the number of connections
	///  aggressive:true  -> do actual compression to determine resulting actual page size
	pub fn try_add_connection_to_page(
		&self,
		page: &mut GraphPage,
		connection_id: &DsnpUserId,
		aggressive: bool,
		keys: &Option<ResolvedKeyPair>,
		dsnp_version_config: Option<&DsnpVersionConfig>,
	) -> Result<()> {
		let connection_type = self.get_connection_type();
		let max_connections_per_page =
			*PAGE_CAPACITY_MAP.get(&connection_type).unwrap_or_else(|| {
				let mut capacities: Vec<&usize> = PAGE_CAPACITY_MAP.values().collect();
				capacities.sort();
				capacities.first().unwrap() // default: return smallest capacity value
			});

		if !aggressive {
			if page.connections().len() >= max_connections_per_page {
				return Err(Error::msg("Page trivially full"))
			}

			if let Err(e) = page.add_connection(connection_id) {
				return Err(e)
			}

			return Ok(())
		}

		if connection_type.privacy_type() == PrivacyType::Private &&
			(dsnp_version_config.is_none() || keys.is_none())
		{
			return Err(Error::msg("Missing encryption key or config"))
		}

		let max_page_size = self.environment.get_config().max_graph_page_size_bytes as usize;
		// Max page fullness leaves room for 2 uncompressed connections
		let max_page_fullness = max_page_size -
			(2 as usize *
				(std::mem::size_of::<DsnpGraphEdge>() + std::mem::size_of::<DsnpPrid>()));

		let mut temp_page = page.clone();
		if let Err(e) = temp_page.add_connection(connection_id) {
			return Err(e)
		}

		let page_blob = match connection_type {
			ConnectionType::Follow(PrivacyType::Public) |
			ConnectionType::Friendship(PrivacyType::Public) => temp_page.to_public_page_data(),
			ConnectionType::Follow(PrivacyType::Private) => {
				let keys = keys.as_ref().unwrap();
				let dsnp_version_config = dsnp_version_config.unwrap();
				temp_page.clear_prids();
				temp_page.to_private_page_data(dsnp_version_config, &keys)
			},
			ConnectionType::Friendship(PrivacyType::Private) => {
				let keys = keys.as_ref().unwrap();
				let dsnp_version_config = dsnp_version_config.unwrap();
				self.apply_prids(&mut temp_page, &vec![], &keys)
					.expect("Error applying prids to page");
				temp_page.to_private_page_data(dsnp_version_config, &keys)
			},
		};

		match page_blob {
			Ok(blob) =>
				if blob.content.len() >= max_page_fullness {
					Err(Error::msg("Page aggressively full"))
				} else {
					if let Err(e) = page.add_connection(connection_id) {
						return Err(e)
					}

					Ok(())
				},
			Err(e) => Err(e),
		}
	}
}

/// Macro to get an iterator to all connections across all GraphPages
/// within a Graph.
#[macro_export]
macro_rules! iter_graph_connections {
	( $x:expr ) => {{
		$x.pages()
			.inner()
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
		dsnp::{
			dsnp_configs::{KeyPairType, PublicKeyType},
			pseudo_relationship_identifier::PridProvider,
		},
		graph::{
			key_manager::{UserKeyManager, UserKeyProvider},
			shared_state_manager::{PublicKeyProvider, SharedStateManager},
		},
		tests::{
			helpers::{
				avro_public_payload, create_empty_test_graph, create_test_graph,
				create_test_ids_and_page, GraphPageBuilder, KeyDataBuilder, PageDataBuilder,
				INNER_TEST_DATA,
			},
			mocks::MockUserKeyManager,
		},
		util::time::time_in_ksecs,
	};
	use dryoc::keypair::StackKeyPair;
	use dsnp_graph_config::{DsnpVersion, GraphKeyType, ALL_CONNECTION_TYPES};
	use ntest::*;
	#[allow(unused_imports)]
	use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
	use std::collections::HashMap;

	#[test]
	fn new_graph_is_empty() {
		let env = Environment::Mainnet;
		let user_id = 3;
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(ConnectionType::Follow(PrivacyType::Private))
			.expect("should exist");
		let graph = Graph::new(
			env,
			user_id,
			schema_id,
			Rc::new(RefCell::from(UserKeyManager::new(
				user_id,
				Rc::new(RefCell::from(SharedStateManager::new())),
			))),
		);
		assert_eq!(graph.pages().inner().is_empty(), true);
	}

	#[test]
	fn graph_len_reports_number_of_connections() {
		let graph = create_test_graph(None);
		assert_eq!(graph.len(), 25);
	}

	#[test]
	fn page_setter_sets_pages() {
		let env = Environment::Mainnet;
		let user_id = 3;
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(ConnectionType::Follow(PrivacyType::Private))
			.expect("should exist");
		let mut pages = PageMap::new();
		for i in 0..=1 {
			let (_, p) = create_test_ids_and_page();
			pages.insert(i, p);
		}
		let mut graph = Graph::new(
			env,
			user_id,
			schema_id,
			Rc::new(RefCell::from(UserKeyManager::new(
				user_id,
				Rc::new(RefCell::from(SharedStateManager::new())),
			))),
		);
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
		let user_id = 3;
		let schema_id = environment
			.get_config()
			.get_schema_id_from_connection_type(CONN_TYPE)
			.expect("should exist");
		let pages: PageMap = (0..=environment.get_config().max_page_id as PageId)
			.map(|page_id: PageId| (page_id, GraphPage::new(PRIV_TYPE, page_id)))
			.collect();
		let graph = Graph {
			environment,
			schema_id, // doesn't matter which type
			user_id,
			pages,
			user_key_manager: Rc::new(RefCell::from(UserKeyManager::new(
				user_id,
				Rc::new(RefCell::from(SharedStateManager::new())),
			))),
		};

		assert_eq!(graph.get_next_available_page_id(), None);
	}

	#[test]
	fn get_next_available_page_id_returns_correct_value() {
		let environment = Environment::Mainnet;
		let user_id = 3;
		const CONN_TYPE: ConnectionType = ConnectionType::Follow(PrivacyType::Public);
		const PRIV_TYPE: PrivacyType = CONN_TYPE.privacy_type();
		let schema_id = environment
			.get_config()
			.get_schema_id_from_connection_type(CONN_TYPE)
			.expect("should exist");
		let mut pages: PageMap = (0..environment.get_config().max_page_id as PageId)
			.map(|page_id: PageId| (page_id, GraphPage::new(PRIV_TYPE, page_id)))
			.collect();
		pages.remove(&8);
		let graph = Graph {
			environment,
			schema_id, // doesn't matter which type
			user_id,
			pages,
			user_key_manager: Rc::new(RefCell::from(UserKeyManager::new(
				user_id,
				Rc::new(RefCell::from(SharedStateManager::new())),
			))),
		};

		assert_eq!(graph.get_next_available_page_id(), Some(8));
	}

	#[test]
	fn clear_removes_all_pages() {
		let mut graph = create_test_graph(None);
		assert_eq!(graph.pages.len() > 0, true);
		graph.clear();
		assert_eq!(graph.pages.len(), 0);
	}

	#[test]
	fn import_public_gets_correct_data() {
		let environment = Environment::Mainnet;
		let user_id = 3;
		let connection_type = ConnectionType::Follow(PrivacyType::Public);
		let schema_id = environment
			.get_config()
			.get_schema_id_from_connection_type(connection_type)
			.expect("should exist");
		let mut graph = Graph::new(
			environment,
			user_id,
			schema_id,
			Rc::new(RefCell::from(UserKeyManager::new(
				user_id,
				Rc::new(RefCell::from(SharedStateManager::new())),
			))),
		);
		let blob = PageData { content_hash: 0, page_id: 0, content: avro_public_payload() };
		let pages = vec![blob];

		let _ = graph.import_public(connection_type, &pages);
		assert_eq!(graph.pages.len(), 1);
		let orig_connections: HashSet<DsnpUserId> =
			INNER_TEST_DATA.iter().map(|edge| edge.user_id).collect();
		let imported_connections: HashSet<DsnpUserId> =
			iter_graph_connections!(graph).map(|edge| edge.user_id).collect();
		assert_eq!(orig_connections, imported_connections);
	}

	#[test]
	fn import_private_follow_gets_correct_data() {
		let connection_type = ConnectionType::Follow(PrivacyType::Private);
		let user_id = 3;
		let environment = Environment::Mainnet;
		let schema_id = environment
			.get_config()
			.get_schema_id_from_connection_type(connection_type)
			.expect("should exist");
		let shared_state_manager = Rc::new(RefCell::from(SharedStateManager::new()));
		let user_key_manager =
			Rc::new(RefCell::from(UserKeyManager::new(user_id, shared_state_manager.clone())));

		let mut graph = Graph::new(environment, user_id, schema_id, user_key_manager.clone());
		let raw_key_pair = StackKeyPair::gen();
		let resolved_key =
			ResolvedKeyPair { key_pair: KeyPairType::Version1_0(raw_key_pair.clone()), key_id: 1 };
		let dsnp_config = DsnpVersionConfig::new(DsnpVersion::Version1_0);
		let orig_connections: HashSet<DsnpUserId> =
			INNER_TEST_DATA.iter().map(|edge| edge.user_id).collect();
		let pages = PageDataBuilder::new(connection_type)
			.with_encryption_key(resolved_key.clone())
			.with_page(0, &orig_connections.iter().map(|u| (*u, 0)).collect::<Vec<_>>(), &vec![], 0)
			.build();
		let graph_key_pair = GraphKeyPair {
			key_type: GraphKeyType::X25519,
			secret_key: raw_key_pair.secret_key.to_vec(),
			public_key: raw_key_pair.public_key.to_vec(),
		};
		let dsnp_keys = DsnpKeys {
			keys: KeyDataBuilder::new().with_key_pairs(&vec![graph_key_pair.clone()]).build(),
			dsnp_user_id: user_id,
			keys_hash: 0,
		};
		shared_state_manager
			.borrow_mut()
			.import_dsnp_keys(&dsnp_keys)
			.expect("should succeed");
		user_key_manager
			.borrow_mut()
			.import_key_pairs(vec![graph_key_pair])
			.expect("should succeed");
		let res = graph.import_private(&dsnp_config, connection_type, &pages);

		assert!(res.is_ok());
		assert_eq!(graph.pages.len(), 1);
		let imported_connections: HashSet<DsnpUserId> =
			iter_graph_connections!(graph).map(|edge| edge.user_id).collect();
		assert_eq!(orig_connections, imported_connections);
	}

	#[test]
	fn create_page_with_existing_pageid_fails() {
		let mut graph = create_test_graph(None);

		assert_eq!(graph.create_page(&0, None).is_err(), true);
	}

	#[test]
	fn create_page_succeeds() {
		let environment = Environment::Mainnet;
		let user_id = 3;
		let schema_id = environment
			.get_config()
			.get_schema_id_from_connection_type(ConnectionType::Follow(PrivacyType::Private))
			.expect("should exist");
		let (_, page) = create_test_ids_and_page();
		let mut graph = Graph::new(
			environment,
			user_id,
			schema_id,
			Rc::new(RefCell::from(UserKeyManager::new(
				user_id,
				Rc::new(RefCell::from(SharedStateManager::new())),
			))),
		);

		assert_eq!(graph.create_page(&0, Some(page.clone())).is_ok(), true);
		assert_eq!(page, *graph.get_page(&0).unwrap());
	}

	#[test]
	fn has_connection_returns_false_for_missing_connection() {
		let graph = create_test_graph(None);

		assert_eq!(graph.has_connection(&99), false);
	}

	#[test]
	fn has_connection_returns_true_for_present_connection() {
		let graph = create_test_graph(None);

		assert_eq!(graph.has_connection(&1), true);
	}

	#[test]
	fn find_connection_returns_none_for_nonexistent_connection() {
		let graph = create_test_graph(None);

		assert_eq!(graph.find_connection(&99), None);
	}

	#[test]
	fn find_connections_returns_pageid_of_existing_connection() {
		let graph = create_test_graph(None);

		assert_eq!(graph.find_connection(&1), Some(0));
	}

	#[test]
	fn find_connections_returns_vec_of_pageids() {
		let graph = create_test_graph(None);

		let mut v = graph.find_connections(&vec![1, 5, 24]);
		v.sort();
		assert_eq!(v, vec![0, 1, 4]);
	}

	#[test]
	fn add_connection_duplicate_connection_errors() {
		let mut graph = create_test_graph(None);

		assert_eq!(graph.add_connection_to_page(&4, &0).is_err(), true);
	}

	#[test]
	fn add_connection_to_nonexistent_page_adds_new_page() {
		let mut graph = create_test_graph(None);
		let page_to_add: PageId = 99;

		assert_eq!(graph.pages().inner().contains_key(&page_to_add), false);
		let _ = graph.add_connection_to_page(&page_to_add, &12345);
		assert_eq!(graph.pages().inner().contains_key(&page_to_add), true);
	}

	#[test]
	fn add_connection_succeeds() {
		let mut graph = create_test_graph(None);

		let _ = graph.add_connection_to_page(&4, &99);
		assert_eq!(graph.find_connection(&99), Some(4));
	}

	#[test]
	fn remove_connection_returns_none_for_not_found() {
		let mut graph = create_test_graph(None);

		let result = graph.remove_connection(&99);
		assert_eq!(result.unwrap().is_none(), true);
	}

	#[test]
	fn remove_connection_returns_pageid_of_removed_connection() {
		let mut graph = create_test_graph(None);

		let result = graph.remove_connection(&5);
		assert_eq!(result.unwrap(), Some(1));
	}

	#[test]
	fn graph_iterator_should_iterate_over_all_connections() {
		let graph = create_test_graph(None);
		let mut test_connections: Vec<DsnpUserId> = (0..25).map(|i| i as DsnpUserId).collect();
		test_connections.sort();

		let mut graph_connections: Vec<DsnpUserId> =
			iter_graph_connections!(graph).map(|edge| edge.user_id).collect();
		graph_connections.sort();
		assert_eq!(test_connections, graph_connections);
	}

	fn updates_to_page(updates: &[Update]) -> Vec<PageData> {
		updates
			.iter()
			.filter_map(|u| match u {
				Update::PersistPage { page_id, payload, .. } =>
					Some(PageData { page_id: *page_id, content_hash: 0, content: payload.clone() }),
				_ => None,
			})
			.collect()
	}

	#[test]
	#[timeout(5000)] // let's make sure this terminates successfully
	fn calculate_updates_public_existing_pages_succeeds() {
		// arrange
		let connection_type = ConnectionType::Follow(PrivacyType::Public);
		let ids_per_page = 5;
		let user_id = 3;
		let mut curr_id = 1u64;
		let mut page_builder = GraphPageBuilder::new(connection_type);
		for i in 0..5 {
			let ids: Vec<(DsnpUserId, u64)> =
				(curr_id..(curr_id + ids_per_page)).map(|id| (id, 0)).collect();
			page_builder = page_builder.with_page(i, &ids, &vec![], 0);
			curr_id += ids_per_page;
		}

		let env = Environment::Mainnet;
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(connection_type)
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
			let _ = graph.create_page(&p.page_id(), Some(p)).expect("should create page!");
		}
		let updates = vec![
			UpdateEvent::create_remove(1, graph.schema_id),
			UpdateEvent::create_remove(1 + ids_per_page * 2, graph.schema_id),
			UpdateEvent::create_add(curr_id + 1, graph.schema_id),
			UpdateEvent::create_add(curr_id + 2, graph.schema_id),
		];

		// act
		let updates =
			graph.calculate_updates(&DsnpVersionConfig::new(DsnpVersion::Version1_0), &updates);

		// assert
		assert!(updates.is_ok());
		let updates = updates.unwrap();

		assert_eq!(updates.len(), 2);
		graph
			.import_public(connection_type, &updates_to_page(&updates))
			.expect("should import");

		let removed_connection_1 = graph.find_connection(&1);
		let removed_connection_2 = graph.find_connection(&(1 + ids_per_page * 2));
		assert!(removed_connection_1.is_none());
		assert!(removed_connection_2.is_none());

		let added_connection_1 = graph.find_connection(&(curr_id + 1));
		let added_connection_2 = graph.find_connection(&(curr_id + 2));
		assert_eq!(added_connection_1, Some(0));
		assert_eq!(added_connection_2, Some(0));
	}

	#[test]
	#[timeout(5000)] // let's make sure this terminates successfully
	fn calculate_updates_public_adding_new_page_should_succeed() {
		// arrange
		let connection_type = ConnectionType::Follow(PrivacyType::Public);
		let ids_per_page =
			*PAGE_CAPACITY_MAP.get(&connection_type).expect("Missing page capacity") as u64;
		let user_id = 3;
		let mut curr_id = 1u64;
		let mut page_builder = GraphPageBuilder::new(connection_type);
		// First trivially fill the page
		for i in 0..2 {
			let ids: Vec<(DsnpUserId, u64)> =
				(curr_id..(curr_id + ids_per_page)).map(|u| (u, 0)).collect();
			page_builder = page_builder.with_page(i, &ids, &vec![], 0);
			curr_id += ids_per_page;
		}

		// Now we need to make sure the page is aggressively full
		let dsnp_version_config = DsnpVersionConfig::new(DsnpVersion::Version1_0);
		let env = Environment::Mainnet;
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(connection_type)
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

		for mut p in page_builder.build() {
			// let page_id = p.page_id();
			// let page = graph.create_page(&page_id, Some(p.clone())).expect("should create page!");

			while graph
				.try_add_connection_to_page(
					&mut p,
					&curr_id,
					true,
					&None,
					Some(&dsnp_version_config),
				)
				.is_ok()
			{
				curr_id += 1;
			}

			graph.create_page(&p.page_id(), Some(p.clone())).expect("should create page!");
		}

		let updates = vec![
			UpdateEvent::create_add(curr_id + 1, graph.schema_id),
			UpdateEvent::create_add(curr_id + 2, graph.schema_id),
		];

		// act
		let updates = graph.calculate_updates(&dsnp_version_config, &updates);

		// assert
		assert!(updates.is_ok());
		let updates = updates.unwrap();
		println!("Updates = {:?}", updates);

		assert_eq!(updates.len(), 1);
		graph
			.import_public(connection_type, &updates_to_page(&updates))
			.expect("should import");

		let added_connection_1 = graph.find_connection(&(curr_id + 1));
		let added_connection_2 = graph.find_connection(&(curr_id + 2));
		assert_eq!(added_connection_1, Some(2));
		assert_eq!(added_connection_2, Some(2));
	}

	#[test]
	#[timeout(5000)] // let's make sure this terminates successfully
	fn calculate_updates_private_follow_pages_should_succeed() {
		// arrange
		let connection_type = ConnectionType::Follow(PrivacyType::Private);
		let ids_per_page = 5;
		let user_id = 3;
		let mut curr_id = 1u64;
		let key =
			ResolvedKeyPair { key_id: 1, key_pair: KeyPairType::Version1_0(StackKeyPair::gen()) };
		let mut page_builder = GraphPageBuilder::new(connection_type);
		for i in 0..5 {
			let ids: Vec<_> = (curr_id..(curr_id + ids_per_page)).map(|u| (u, 0)).collect();
			let prids: Vec<_> =
				ids.iter().map(|(id, _)| DsnpPrid::new(&id.to_le_bytes())).collect();
			page_builder = page_builder.with_page(i, &ids, &prids, 0);
			curr_id += ids_per_page;
		}

		let env = Environment::Mainnet;
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(connection_type)
			.expect("should exist");
		let shared_state = Rc::new(RefCell::from(SharedStateManager::new()));
		let user_key = Rc::new(RefCell::from(UserKeyManager::new(user_id, shared_state.clone())));
		let mut graph = Graph::new(env, user_id, schema_id, user_key.clone());
		for p in page_builder.build() {
			let _ = graph.create_page(&p.page_id(), Some(p)).expect("should create page!");
		}
		shared_state
			.borrow_mut()
			.import_keys_test(
				user_id,
				&vec![DsnpPublicKey {
					key_id: Some(key.key_id),
					key: key.key_pair.get_public_key_raw(),
				}],
				0,
			)
			.expect("should insert keys");
		user_key
			.borrow_mut()
			.import_key_pairs(vec![GraphKeyPair {
				key_type: GraphKeyType::X25519,
				public_key: key.key_pair.get_public_key_raw(),
				secret_key: key.key_pair.get_secret_key_raw(),
			}])
			.expect("should import user keys");

		let updates = vec![
			UpdateEvent::create_remove(1, graph.schema_id),
			UpdateEvent::create_add(curr_id + 1, graph.schema_id),
		];

		// act
		let updates =
			graph.calculate_updates(&DsnpVersionConfig::new(DsnpVersion::Version1_0), &updates);

		// assert
		assert!(updates.is_ok());
		let updates = updates.unwrap();

		assert_eq!(updates.len(), 1);
		graph
			.import_private(
				&DsnpVersionConfig::new(DsnpVersion::Version1_0),
				connection_type,
				&updates_to_page(&updates),
			)
			.expect("should import");

		let removed_connection_1 = graph.find_connection(&1);
		assert!(removed_connection_1.is_none());

		let added_connection_1 = graph.find_connection(&(curr_id + 1));
		assert_eq!(added_connection_1, Some(0));
	}

	#[test]
	#[timeout(5000)] // let's make sure this terminates successfully
	fn calculate_updates_private_friendship_pages_should_succeed() {
		// arrange
		let connection_type = ConnectionType::Friendship(PrivacyType::Private);
		let ids_per_page = 5;
		let user_id = 1000;
		let mut curr_id = 1u64;
		let mut page_builder = GraphPageBuilder::new(connection_type);
		let mut key_mapper = HashMap::new();
		let shared_state = Rc::new(RefCell::from(SharedStateManager::new()));
		let owner_key =
			ResolvedKeyPair { key_id: 1, key_pair: KeyPairType::Version1_0(StackKeyPair::gen()) };
		let removed_friend_user_id: DsnpUserId = 3;
		let non_stale_friend_user_id: DsnpUserId = 4;
		for i in 0..5 {
			let ids: Vec<(DsnpUserId, u64)> = (curr_id..(curr_id + ids_per_page))
				.map(|u| if u == removed_friend_user_id { (u, 0) } else { (u, time_in_ksecs()) })
				.collect();
			ids.iter().for_each(|(id, _since)| {
				key_mapper.insert(
					*id,
					DsnpPublicKey { key_id: Some(1), key: StackKeyPair::gen().public_key.to_vec() },
				);

				let public_key: PublicKeyType = key_mapper.get(id).unwrap().try_into().unwrap();
				let mut prid = DsnpPrid::create_prid(
					*id,
					user_id,
					&owner_key.clone().key_pair.into(),
					&public_key,
				)
				.unwrap();

				if *id == removed_friend_user_id || *id == non_stale_friend_user_id {
					// setting wrong prid
					prid = DsnpPrid::new(&[1u8, 2, 3, 4, 5, 6, 7, 8]);
				}
				shared_state
					.borrow_mut()
					.import_prids_test(*id, &vec![prid], 1)
					.expect("should import prid");
			});
			let prids: Vec<_> = ids
				.iter()
				.map(|(id, _since)| {
					let public_key: PublicKeyType = key_mapper.get(id).unwrap().try_into().unwrap();
					DsnpPrid::create_prid(
						user_id,
						*id,
						&owner_key.clone().key_pair.into(),
						&public_key,
					)
					.unwrap()
				})
				.collect();
			page_builder = page_builder.with_page(i, &ids, &prids, 0);
			curr_id += ids_per_page;
		}

		let env = Environment::Mainnet;
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(connection_type)
			.expect("should exist");
		let user_key = Rc::new(RefCell::from(UserKeyManager::new(user_id, shared_state.clone())));
		let mut graph = Graph::new(env, user_id, schema_id, user_key.clone());
		for p in page_builder.build() {
			let _ = graph.create_page(&p.page_id(), Some(p)).expect("should create page!");
		}
		let mut dsnp_keys = vec![(
			user_id,
			DsnpPublicKey {
				key_id: Some(owner_key.key_id),
				key: owner_key.key_pair.get_public_key_raw(),
			},
		)];
		let other_keys: Vec<_> = key_mapper.iter().map(|(a, b)| (*a, b.clone())).collect();
		dsnp_keys.extend_from_slice(&other_keys);
		// add public key for new connection
		dsnp_keys.push((
			curr_id + 1,
			DsnpPublicKey { key_id: Some(1), key: StackKeyPair::gen().public_key.to_vec() },
		));
		for (user, key) in dsnp_keys {
			shared_state
				.borrow_mut()
				.import_keys_test(user, &vec![key], 0)
				.expect("should insert keys");
		}
		user_key
			.borrow_mut()
			.import_key_pairs(vec![GraphKeyPair {
				key_type: GraphKeyType::X25519,
				public_key: owner_key.key_pair.get_public_key_raw(),
				secret_key: owner_key.key_pair.get_secret_key_raw(),
			}])
			.expect("should import user keys");

		let updates = vec![
			UpdateEvent::create_remove(1, graph.schema_id),
			UpdateEvent::create_add(curr_id + 1, graph.schema_id),
		];

		// act
		let updates =
			graph.calculate_updates(&DsnpVersionConfig::new(DsnpVersion::Version1_0), &updates);

		// assert
		assert!(updates.is_ok());
		let updates = updates.unwrap();

		assert_eq!(updates.len(), 1);
		graph
			.import_private(
				&DsnpVersionConfig::new(DsnpVersion::Version1_0),
				connection_type,
				&updates_to_page(&updates),
			)
			.expect("should import");

		let removed_connection_1 = graph.find_connection(&1);
		assert!(removed_connection_1.is_none());

		let added_connection_1 = graph.find_connection(&(curr_id + 1));
		assert_eq!(added_connection_1, Some(0));

		let removed_connection_2 = graph.find_connection(&removed_friend_user_id);
		assert!(removed_connection_2.is_none());

		let should_not_be_removed = graph.find_connection(&non_stale_friend_user_id);
		assert_eq!(should_not_be_removed, Some(0));
	}

	#[test]
	fn get_one_sided_friendships_should_return_expected_connections() {
		// arrange
		let connection_type = ConnectionType::Friendship(PrivacyType::Private);
		let env = Environment::Mainnet;
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(connection_type)
			.expect("should exist");
		let mut key_manager = MockUserKeyManager::new();
		let max_connections = PAGE_CAPACITY_MAP.get(&connection_type).unwrap();
		let ids: Vec<(DsnpUserId, u64)> =
			(1..*max_connections as u64 as DsnpUserId).map(|u| (u, 0)).collect();
		let verifications: Vec<_> = ids.iter().map(|(id, _)| (*id, Some(true))).collect();
		key_manager.register_verifications(&verifications);
		// register one sided connections
		key_manager.register_verifications(&vec![(1, Some(false)), (2, Some(false))]);
		let mut graph = Graph::new(env, 1000, schema_id, Rc::new(RefCell::from(key_manager)));
		for p in GraphPageBuilder::new(connection_type)
			.with_page(1, &ids, &vec![DsnpPrid::new(&[0, 1, 2, 3, 4, 5, 6, 7]); ids.len()], 0)
			.build()
		{
			let _ = graph.create_page(&p.page_id(), Some(p)).expect("should create page!");
		}

		// act
		let one_sided = graph.get_one_sided_friendships();

		// assert
		assert!(one_sided.is_ok());
		let one_sided = one_sided.unwrap();
		assert_eq!(
			one_sided,
			vec![DsnpGraphEdge { user_id: 1, since: 0 }, DsnpGraphEdge { user_id: 2, since: 0 }]
		);
	}

	#[test]
	fn private_friendship_functions_should_fail_for_non_private_friendship_graphs() {
		let env = Environment::Mainnet;
		let failures = vec![
			ConnectionType::Friendship(PrivacyType::Public),
			ConnectionType::Follow(PrivacyType::Private),
			ConnectionType::Follow(PrivacyType::Public),
		];

		for connection_type in failures {
			// arrange
			let schema_id = env
				.get_config()
				.get_schema_id_from_connection_type(connection_type)
				.expect("should exist");
			let graph = Graph::new(
				env.clone(),
				1000,
				schema_id,
				Rc::new(RefCell::from(MockUserKeyManager::new())),
			);

			// act
			let one_sided = graph.get_one_sided_friendships();
			let prids = graph.apply_prids(
				&mut GraphPage::new(connection_type.privacy_type(), 1),
				&vec![],
				&ResolvedKeyPair {
					key_id: 1,
					key_pair: KeyPairType::Version1_0(StackKeyPair::gen()),
				},
			);

			// assert
			assert!(one_sided.is_err());
			assert!(prids.is_err());
		}
	}

	#[test]
	fn get_one_sided_friendships_with_key_related_errors_should_fail() {
		// arrange
		let connection_type = ConnectionType::Friendship(PrivacyType::Private);
		let env = Environment::Mainnet;
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(connection_type)
			.expect("should exist");
		let mut key_manager = MockUserKeyManager::new();
		let max_connections = PAGE_CAPACITY_MAP.get(&connection_type).unwrap();
		let ids: Vec<(DsnpUserId, u64)> =
			(1..*max_connections as u64 as DsnpUserId).map(|id| (id, 0)).collect();
		let verifications: Vec<_> = ids.iter().map(|(id, _)| (*id, Some(true))).collect();
		key_manager.register_verifications(&verifications);
		// register failure
		key_manager.register_verifications(&vec![(2, None)]);
		let mut graph = Graph::new(env, 1000, schema_id, Rc::new(RefCell::from(key_manager)));
		for p in GraphPageBuilder::new(connection_type)
			.with_page(1, &ids, &vec![DsnpPrid::new(&[0, 1, 2, 3, 4, 5, 6, 7]); ids.len()], 0)
			.build()
		{
			let _ = graph.create_page(&p.page_id(), Some(p)).expect("should create page!");
		}

		// act
		let one_sided = graph.get_one_sided_friendships();

		// assert
		assert!(one_sided.is_err());
	}

	#[test]
	fn trivial_add_to_trivially_non_full_page_succeeds() {
		ALL_CONNECTION_TYPES.iter().for_each(|c| {
			let graph = create_empty_test_graph(Some(*c));
			let max_connections_per_page = PAGE_CAPACITY_MAP
				.get(c)
				.expect("Connection type missing max connections soft limit");
			let builder = GraphPageBuilder::new(*c).with_page(1, &[], &[], 0);
			let mut pages = builder.build();
			let page = pages.first_mut().expect("Should have created page");

			for i in 1u64..*max_connections_per_page as u64 {
				assert_eq!(
					graph.try_add_connection_to_page(page, &i, false, &None, None).is_ok(),
					true,
					"Testing soft connection limit for {:?}",
					c,
				);
			}
		});
	}

	#[test]
	fn trivial_add_to_trivially_full_page_fails() {
		ALL_CONNECTION_TYPES.iter().for_each(|c| {
			let graph = create_empty_test_graph(Some(*c));
			let max_connections_per_page = PAGE_CAPACITY_MAP
				.get(c)
				.expect("Connection type missing max connections soft limit");
			let connections = (0..*max_connections_per_page as u64)
				.map(|id| (id, 0))
				.collect::<Vec<(u64, u64)>>();
			let prids = match c.privacy_type() {
				PrivacyType::Private =>
					connections.iter().clone().map(|(id, _)| DsnpPrid::from(*id)).collect(),
				_ => vec![],
			};
			let mut pages = GraphPageBuilder::new(*c).with_page(1, &connections, &prids, 0).build();

			let page = pages.first_mut().expect("page should exist");
			let conn_id: u64 = *max_connections_per_page as u64;
			assert_eq!(
				graph.try_add_connection_to_page(page, &conn_id, false, &None, None).is_err(),
				true,
				"Testing soft connection limit for {:?}",
				c
			);
		});
	}

	#[test]
	#[ignore = "todo"]
	fn aggressive_add_to_trivially_non_full_page_succeeds() {}

	#[test]
	#[ignore = "todo"]
	fn aggressive_add_to_aggressively_non_full_page_succeeds() {}

	#[test]
	#[ignore = "todo"]
	fn aggressive_add_to_aggressively_full_page_fails() {}

	#[test]
	fn graph_page_rollback_should_revert_changes_on_graph_and_all_underlying_page() {
		// arrange
		let connection_type = ConnectionType::Friendship(PrivacyType::Private);
		let env = Environment::Mainnet;
		let mut graph =
			Graph::new(env, 1000, 2000, Rc::new(RefCell::from(MockUserKeyManager::new())));
		let mut page_1 = GraphPage::new(connection_type.privacy_type(), 1);
		let connection_dsnp = 900;
		page_1.add_connection(&connection_dsnp).unwrap();
		graph.pages.insert(1, page_1.clone());
		graph.commit();

		let page_1 = graph.pages.get_mut(&1).unwrap();
		page_1.remove_connection(&connection_dsnp).unwrap();
		page_1.add_connection(&500).unwrap();
		let mut page_2 = GraphPage::new(connection_type.privacy_type(), 2);
		page_2.add_connection(&400).unwrap();
		graph.create_page(&2, Some(page_2)).unwrap();

		// act
		graph.rollback();

		// assert
		assert_eq!(graph.pages.len(), 1);
		assert_eq!(graph.pages.get(&1).unwrap().connections().len(), 1);
		assert!(graph.pages.get(&1).unwrap().contains(&connection_dsnp));
	}

	#[test]
	fn force_recalculate_public_should_work_as_expected() {
		// arrange
		let connection_type = ConnectionType::Follow(PrivacyType::Public);
		let env = Environment::Mainnet;
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(connection_type)
			.expect("should exist");
		let user_id = 1000;
		let ids: Vec<_> = (1..50).map(|u| (u, 0)).collect();
		let pages = GraphPageBuilder::new(connection_type).with_page(1, &ids, &vec![], 0).build();
		let mut graph =
			Graph::new(env, user_id, schema_id, Rc::new(RefCell::from(MockUserKeyManager::new())));
		for (i, p) in pages.into_iter().enumerate() {
			let _ = graph.create_page(&(i as PageId), Some(p));
		}
		// act
		let updates = graph.force_recalculate(&DsnpVersionConfig::new(DsnpVersion::Version1_0));

		// assert
		assert!(updates.is_ok());
		let updates = updates.unwrap();
		assert_eq!(updates.len(), 1);
		assert!(matches!(updates.get(0).unwrap(), Update::PersistPage { .. }));
	}

	#[test]
	fn force_recalculate_private_follow_should_work_as_expected() {
		// arrange
		let connection_type = ConnectionType::Follow(PrivacyType::Private);
		let env = Environment::Mainnet;
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(connection_type)
			.expect("should exist");
		let user_id = 1000;
		let ids: Vec<_> = (1..50).map(|u| (u, 0)).collect();
		let pages = GraphPageBuilder::new(connection_type).with_page(1, &ids, &vec![], 0).build();
		let key =
			ResolvedKeyPair { key_id: 1, key_pair: KeyPairType::Version1_0(StackKeyPair::gen()) };
		let mut key_manager = MockUserKeyManager::new();
		key_manager.register_key(user_id, &key);
		let mut graph = Graph::new(env, user_id, schema_id, Rc::new(RefCell::from(key_manager)));
		for (i, p) in pages.into_iter().enumerate() {
			let _ = graph.create_page(&(i as PageId), Some(p));
		}
		// act
		let updates = graph.force_recalculate(&DsnpVersionConfig::new(DsnpVersion::Version1_0));

		// assert
		assert!(updates.is_ok());
		let updates = updates.unwrap();
		assert_eq!(updates.len(), 1);
		assert!(matches!(updates.get(0).unwrap(), Update::PersistPage { .. }));
	}

	#[test]
	fn force_recalculate_private_friendship_should_work_as_expected() {
		// arrange
		let connection_type = ConnectionType::Friendship(PrivacyType::Private);
		let env = Environment::Mainnet;
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(connection_type)
			.expect("should exist");
		let user_id = 1000;
		let ids: Vec<_> = (1..50).map(|u| (u, 0)).collect();
		let pages = GraphPageBuilder::new(connection_type)
			.with_page(1, &ids, &vec![DsnpPrid::new(&[0, 1, 2, 3, 4, 5, 6, 7]); ids.len()], 0)
			.build();
		let key =
			ResolvedKeyPair { key_id: 1, key_pair: KeyPairType::Version1_0(StackKeyPair::gen()) };
		let mut key_manager = MockUserKeyManager::new();
		key_manager.register_key(user_id, &key);
		let verifications: Vec<_> = ids.iter().map(|(id, _)| (*id, Some(true))).collect();
		key_manager.register_verifications(&verifications);
		let mut graph = Graph::new(env, user_id, schema_id, Rc::new(RefCell::from(key_manager)));
		for (i, p) in pages.into_iter().enumerate() {
			let _ = graph.create_page(&(i as PageId), Some(p));
		}
		// act
		let updates = graph.force_recalculate(&DsnpVersionConfig::new(DsnpVersion::Version1_0));

		// assert
		assert!(updates.is_ok());
		let updates = updates.unwrap();
		assert_eq!(updates.len(), 1);
		assert!(matches!(updates.get(0).unwrap(), Update::PersistPage { .. }));
	}
}
