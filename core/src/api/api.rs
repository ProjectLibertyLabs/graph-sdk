use crate::dsnp::{
	api_types::{
		Connection, ConnectionType, DsnpKeys, ExportBundle, ImportBundle, PrivacyType, PublicKey,
	},
	dsnp_types::DsnpUserId,
	encryption::EncryptionBehavior,
	graph_page::UserGraph,
};
use anyhow::{Error, Result};
use std::{cmp::min, collections::HashMap};

const MAX_GRAPH_USERS_DEFAULT: usize = 1000;

#[allow(non_snake_case)]
pub struct GraphState<const MAX_USERS: usize = MAX_GRAPH_USERS_DEFAULT> {
	user_map: HashMap<DsnpUserId, UserGraph>,
}

pub trait GraphAPI<E: EncryptionBehavior> {
	/// Create a new, empty user graph
	fn add_user_graph(&mut self, user_id: &DsnpUserId) -> Result<&mut UserGraph>;

	/// Remove the user graph from an SDK instance
	fn remove_user_graph(&mut self, user_id: &DsnpUserId);

	/// Import raw data retrieved from the blockchain into a user graph.
	/// Will overwrite any existing graph data for the user,
	/// but pending updates will be preserved.
	fn import_user_data(&mut self, payload: ImportBundle<E>) -> Result<()>;

	/// Calculate the necessary page updates for a user's graph, and
	/// return as a map of pages to be updated and/or removed
	fn export_user_updates(
		&mut self,
		user_id: &DsnpUserId,
		connection_keys: &Vec<DsnpKeys<E>>,
		encryption_key: (u64, &PublicKey<E>),
	) -> Result<Vec<ExportBundle>>;

	/// Add the connection to the list of pending additions for the user
	fn add_connection_for_user(user_id: &DsnpUserId, connection: &Connection) -> Result<()>;

	/// Add an indication that the connection is pending removal for the user
	fn remove_connection_from_user(user_id: &DsnpUserId, connection: &Connection);

	/// Get a list of all connections of the indicated type for the user
	fn get_connections_for_user_graph(
		user_id: &DsnpUserId,
		connection_type: &ConnectionType,
		include_pending: bool,
	) -> Vec<Connection>;
}

impl<const MAX_USERS: usize> GraphState<MAX_USERS> {
	pub fn new() -> Self {
		Self { user_map: HashMap::<DsnpUserId, UserGraph>::new() }
	}

	pub fn with_capacity(capacity: usize) -> Self {
		let size = min(capacity, MAX_USERS);
		Self { user_map: HashMap::<DsnpUserId, UserGraph>::with_capacity(size) }
	}
}

impl<E: EncryptionBehavior, const M: usize> GraphAPI<E> for GraphState<M> {
	/// Create a new, empty user graph
	fn add_user_graph(&mut self, user_id: &DsnpUserId) -> Result<&mut UserGraph> {
		if self.user_map.len() >= M {
			return Err(Error::msg("GraphState instance full"))
		}

		if self.user_map.contains_key(user_id) {
			return Err(Error::msg(
				"Detected attempt to create a duplicate UserGraph instance for a user",
			))
		}

		self.user_map.insert(*user_id, UserGraph::new(user_id));
		match self.user_map.get_mut(user_id) {
			Some(graph) => Ok(graph),
			None => Err(Error::msg("Unexpected error retrieving user graph")),
		}
	}

	/// Remove the user graph from an instance
	fn remove_user_graph(&mut self, user_id: &DsnpUserId) {
		self.user_map.remove(user_id);
	}

	/// Import raw data retrieved from the blockchain into a user graph.
	/// Will overwrite any existing graph data for the user,
	/// but pending updates will be preserved.
	fn import_user_data(&mut self, payload: ImportBundle<E>) -> Result<()> {
		let user_graph = match self.user_map.get_mut(&payload.dsnp_user_id) {
			Some(graph) => graph,
			None => <GraphState<M> as GraphAPI<E>>::add_user_graph(self, &payload.dsnp_user_id)?,
		};

		let graph = user_graph.graph_mut(&payload.connection_type);
		graph.clear();

		match payload.connection_type.privacy_type() {
			PrivacyType::Public => graph.import_public(payload),
			PrivacyType::Private =>
			// An import of a private graph with an empty key list is an "opaque" import
				if payload.keys.is_empty() {
					graph.import_opaque(payload)
				} else {
					graph.import_private(payload)
				},
		}?;

		Ok(())
	}

	/// Calculate the necessary page updates for a user's graph, and
	/// return as a map of pages to be updated and/or removed
	fn export_user_updates(
		&mut self,
		user_id: &DsnpUserId,
		connection_keys: &Vec<DsnpKeys<E>>,
		encryption_key: (u64, &PublicKey<E>),
	) -> Result<Vec<ExportBundle>> {
		let user_graph = match self.user_map.get_mut(user_id) {
			None => Err(Error::msg("User not found for graph export")),
			Some(graph) => Ok(graph),
		}?;

		user_graph.calculate_updates(connection_keys, encryption_key)
	}

	/// Add the connection to the list of pending additions for the user
	fn add_connection_for_user(_user_id: &DsnpUserId, _connection: &Connection) -> Result<()> {
		todo!();
	}

	/// Add an indication that the connection is pending removal for the user
	fn remove_connection_from_user(_user_id: &DsnpUserId, _connection: &Connection) {
		todo!();
	}

	/// Get a list of all connections of the indicated type for the user
	fn get_connections_for_user_graph(
		_user_id: &DsnpUserId,
		_connection_type: &ConnectionType,
		_include_pending: bool,
	) -> Vec<Connection> {
		todo!();
	}
}
