use crate::{
	dsnp::{
		api_types::{
			Connection, ConnectionType, DsnpKeys, ExportBundle, ImportBundle, PrivacyType,
			PublicKey,
		},
		dsnp_types::{DsnpGraphEdge, DsnpUserId},
		encryption::EncryptionBehavior,
	},
	graph::{updates::UpdateEvent, user::UserGraph},
	iter_graph_connections,
	util::time::time_in_ksecs,
};
use anyhow::{Error, Result};
use std::{cmp::min, collections::HashMap, marker::PhantomData};

const MAX_GRAPH_USERS_DEFAULT: usize = 1000;

#[allow(non_snake_case)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GraphState<E: EncryptionBehavior, const MAX_USERS: usize = MAX_GRAPH_USERS_DEFAULT> {
	_encryption: PhantomData<E>,
	user_map: HashMap<DsnpUserId, UserGraph>,
}

pub trait GraphAPI<E: EncryptionBehavior> {
	/// Check if graph state contains a user
	fn contains_user(&self, user_id: &DsnpUserId) -> bool;

	/// Return number of users in the current graph state
	fn len(&self) -> usize;

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
	fn add_connection_for_user(
		&mut self,
		user_id: &DsnpUserId,
		connection: &Connection,
	) -> Result<()>;

	/// Add an indication that the connection is pending removal for the user
	fn remove_connection_from_user(
		&mut self,
		user_id: &DsnpUserId,
		connection: &Connection,
	) -> Result<()>;

	/// Get a list of all connections of the indicated type for the user
	fn get_connections_for_user_graph(
		&self,
		user_id: &DsnpUserId,
		connection_type: &ConnectionType,
		include_pending: bool,
	) -> Result<Vec<DsnpGraphEdge>>;
}

impl<const MAX_USERS: usize, E: EncryptionBehavior> GraphState<E, MAX_USERS> {
	pub fn new() -> Self {
		Self { _encryption: PhantomData, user_map: HashMap::<DsnpUserId, UserGraph>::new() }
	}

	pub fn with_capacity(capacity: usize) -> Self {
		let size = min(capacity, MAX_USERS);
		Self {
			_encryption: PhantomData,
			user_map: HashMap::<DsnpUserId, UserGraph>::with_capacity(size),
		}
	}

	pub fn capacity(&self) -> usize {
		MAX_USERS
	}
}

impl<E: EncryptionBehavior, const M: usize> GraphAPI<E> for GraphState<E, M> {
	/// Check if graph state contains a user
	fn contains_user(&self, user_id: &DsnpUserId) -> bool {
		self.user_map.contains_key(user_id)
	}

	/// Return number of users in the current graph state
	fn len(&self) -> usize {
		self.user_map.len()
	}

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
			None => self.add_user_graph(&payload.dsnp_user_id)?,
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
	fn add_connection_for_user(
		&mut self,
		user_id: &DsnpUserId,
		connection: &Connection,
	) -> Result<()> {
		let user_graph = match self.user_map.get_mut(user_id) {
			Some(g) => g,
			None => return Err(Error::msg("user graph not found in state")),
		};

		user_graph.update_tracker.register_update(&UpdateEvent::create_add(
			connection.dsnp_user_id,
			connection.connection_type,
		))
	}

	/// Add an indication that the connection is pending removal for the user
	fn remove_connection_from_user(
		&mut self,
		user_id: &DsnpUserId,
		connection: &Connection,
	) -> Result<()> {
		let graph = match self.user_map.get_mut(user_id) {
			Some(g) => g,
			None => return Err(Error::msg("user graph not found in state")),
		};
		graph.update_tracker.register_update(&UpdateEvent::create_remove(
			connection.dsnp_user_id,
			connection.connection_type,
		))
	}

	/// Get a list of all connections of the indicated type for the user
	fn get_connections_for_user_graph(
		&self,
		user_id: &DsnpUserId,
		connection_type: &ConnectionType,
		include_pending: bool,
	) -> Result<Vec<DsnpGraphEdge>> {
		let user_graph = match self.user_map.get(user_id) {
			Some(graph) => graph,
			None => return Err(Error::msg("user not present in graph state")),
		};

		let graph = user_graph.graph(connection_type);
		let mut connections: Vec<DsnpGraphEdge> = iter_graph_connections!(graph).cloned().collect();

		if include_pending {
			user_graph
				.update_tracker
				.get_updates_for_connection_type(*connection_type)
				.unwrap_or(&Vec::<UpdateEvent>::new())
				.iter()
				.cloned()
				.for_each(|event| match event {
					UpdateEvent::Add { dsnp_user_id, .. } =>
						if !connections.iter().map(|c| c.user_id).any(|id| id == dsnp_user_id) {
							connections.push(DsnpGraphEdge {
								user_id: dsnp_user_id,
								since: time_in_ksecs(),
							})
						},
					UpdateEvent::Remove { dsnp_user_id, .. } =>
						connections.retain(|c| c.user_id != dsnp_user_id),
				});
		}

		Ok(connections)
	}
}

#[cfg(test)]
mod test {
	use crate::dsnp::encryption::SealBox;

	use super::*;
	const TEST_CAPACITY: usize = 10;

	type TestGraphState<const M: usize = TEST_CAPACITY> = GraphState<SealBox, M>;

	#[test]
	fn new_graph_state_with_capacity_sets_initial_hash_map_capacity() {
		let capacity: usize = 5;
		let new_state = TestGraphState::<TEST_CAPACITY>::with_capacity(capacity);
		assert!(new_state.user_map.capacity() >= capacity);
	}

	#[test]
	fn new_graph_state_with_capacity_caps_initial_hash_map_capacity() {
		let new_state = TestGraphState::<TEST_CAPACITY>::with_capacity(TEST_CAPACITY * 2);
		assert!(new_state.user_map.capacity() >= TEST_CAPACITY);
	}

	#[test]
	fn graph_state_capacity() {
		let state: TestGraphState = TestGraphState::new();
		assert_eq!(state.capacity(), TEST_CAPACITY);
	}

	#[test]
	fn graph_contains_false() {
		let state: TestGraphState = TestGraphState::new();
		assert!(!state.contains_user(&0));
	}

	#[test]
	fn graph_contains_true() {
		let mut state: TestGraphState = TestGraphState::new();
		let _ = state.add_user_graph(&0);
		assert!(state.contains_user(&0));
	}

	#[test]
	fn graph_len() {
		let mut state: TestGraphState = TestGraphState::new();
		let _ = state.add_user_graph(&0);
		assert_eq!(state.len(), 1);
		let _ = state.add_user_graph(&1);
		assert_eq!(state.len(), 2);
	}

	#[test]
	fn add_user_errors_if_graph_state_full() {
		let mut state = TestGraphState::<1>::new();
		let _ = state.add_user_graph(&0);
		assert!(state.add_user_graph(&1).is_err());
	}

	#[test]
	fn add_duplicate_user_errors() {
		let mut state: TestGraphState = TestGraphState::new();
		let _ = state.add_user_graph(&0);
		assert!(state.add_user_graph(&0).is_err());
	}
	#[test]
	fn add_user_success() -> Result<()> {
		let mut state: TestGraphState = TestGraphState::new();

		state.add_user_graph(&0)?;
		Ok(())
	}

	#[test]
	fn remove_user_success() {
		let mut state: TestGraphState = TestGraphState::new();
		let _ = state.add_user_graph(&0);
		let _ = state.add_user_graph(&1);
		state.remove_user_graph(&0);
		assert_eq!(state.len(), 1);
		assert!(!state.contains_user(&0));
		assert!(state.contains_user(&1));
	}

	#[test]
	fn remove_nonexistent_user_noop() {
		let mut state: TestGraphState = TestGraphState::new();
		let _ = state.add_user_graph(&0);
		let _ = state.add_user_graph(&1);
		let state_copy = state.clone();
		state.remove_user_graph(&99);
		assert_eq!(state, state_copy);
	}

	#[test]
	#[ignore = "todo"]
	fn import_user_data() {}

	#[test]
	#[ignore = "todo"]
	fn export_user_updates() {}

	#[test]
	fn add_duplicate_connection_for_user_errors() {
		let connection = Connection {
			connection_type: ConnectionType::Follow(PrivacyType::Private),
			dsnp_user_id: 1,
		};

		let mut state: TestGraphState = TestGraphState::new();
		let _ = state.add_user_graph(&0);
		assert!(state.add_connection_for_user(&0, &connection).is_ok());
		assert!(state.add_connection_for_user(&0, &connection).is_err());
	}

	#[test]
	fn add_connection_for_nonexistent_user_errors() {
		let mut state: TestGraphState = TestGraphState::new();
		assert!(state
			.add_connection_for_user(
				&0,
				&Connection {
					dsnp_user_id: 1,
					connection_type: ConnectionType::Follow(PrivacyType::Private),
				},
			)
			.is_err());
	}

	#[test]
	fn remove_connection_for_user_twice_errors() {
		let connection: Connection = Connection {
			dsnp_user_id: 1,
			connection_type: ConnectionType::Follow(PrivacyType::Private),
		};
		let mut state: TestGraphState = TestGraphState::new();
		let _ = state.add_user_graph(&0);
		assert!(state.remove_connection_from_user(&0, &connection).is_ok());
		assert!(state.remove_connection_from_user(&0, &connection).is_err());
	}

	#[test]
	fn remove_connection_from_nonexistent_user_errors() {
		let mut state: TestGraphState = TestGraphState::new();
		assert!(state
			.remove_connection_from_user(
				&0,
				&Connection {
					dsnp_user_id: 1,
					connection_type: ConnectionType::Follow(PrivacyType::Private),
				}
			)
			.is_err());
	}

	#[test]
	#[ignore = "todo"]
	fn get_connections_for_user_graph() {}

	#[test]
	#[ignore = "todo"]
	fn get_connections_for_user_graph_with_pending() {}
}
