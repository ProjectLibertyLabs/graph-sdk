//! Graph SDK API allows easy interactions and modification on the social graph
//! # API Design
//! The design of this api is such that it is isolated, meaning all the necessary data should be
//! imported via provided API functions.
//! There are three main steps that encapsulates the typical usage of this SDK
//! - Import all graph related data into the SDK
//! - Read or apply changes to desired graph
//! - Export updates
//!
//! ## Importing data
//! There are two categories of data that can be imported
//! - Raw graph data and public keys stored on Frequency Blockchain
//! - Key-pairs of graph owner to allow encryption/description for private data
//!
//! Following Apis are defined to support importing of data into SDK
//! - `import_users_data` is the main api that provides support to import both type of above mentioned data
//! - `apply_actions` when used with `Connect` action to add a new connection to the graph also allows
//! optional importing of keys associated with new connection
//!
//! ## Graph Interactions
//! After importing the desired graph data we can start reading or updating the graph using following APIs
//! - `contains_user_graph` checks if a specific dsnp user's graph is imported or exists in SDK
//! - `len` returns the number of DSNP users that their graph is imported or exits right now in SDK
//! - `remove_user_graph` allows removal of the graph data from SDK and can be used as a cleanup step
//! - `apply_action` is the main api that allows updating the graph by adding new connections or removing old ones
//! - `get_connections_for_user_graph` exposes imported graph data for a certain user and can be used to read
//! data out SDK
//! - `get_connections_without_keys` the main use-case for this api is for Private Friendship graph and
//! it's to inform the SDK consumer about the connections that their published public keys are not imported.
//! Importing their published public keys are required to determine friendship existence or update the PRId.
//! - `get_one_sided_private_friendship_connections` the main use-case for this api is also for Private
//! Friendship graph and returns broken friendships
//! - `get_public_keys` returns the raw public keys imported for a certain dsnp user.
//! - `deserialize_dsnp_keys` returns deserialized public keys from published on chain DSNP keys without
//! importing them. One use-case might be for wallets to know which key-pairs should be included in
//! `ImportBundle`, when importing graph data.
//!
//! ## Export updates
//! After applying any changes to the graph using API's mentioned in previous step, we can use the
//! following ones to export the updated graph and key data, so it can be stored on chain
//! - `export_updates` this is the main API that returns any updates to the graph or newly added keys
//! - `force_recalculate_graphs` this API can be used to recalculate the graph using the latest published
//! graph key which can be used for encryption or PRId calculation.
//!
//! # Transactional Support
//! All the batch APIs that modify SDK's inner state such as `import_users_data` or `apply_action`
//! are transactional. If one of the imported data or updated actions failed, the inner state will
//! be reverted to before failed call state.

use crate::{
	api::api_types::{
		Action, ActionOptions, Connection, DsnpKeys, ImportBundle, PrivacyType, Update,
	},
	dsnp::{
		dsnp_types::{DsnpGraphEdge, DsnpPublicKey, DsnpUserId},
		reader_writer::DsnpReader,
	},
	frequency::Frequency,
	graph::{
		key_manager::{UserKeyProvider, USER_KEY_MANAGER},
		shared_state_manager::{
			PriProvider, PublicKeyProvider, SharedStateManager, SHARED_STATE_MANAGER,
		},
		updates::UpdateEvent,
		user::UserGraph,
	},
	util::transactional_hashmap::{Transactional, TransactionalHashMap},
};
use dryoc::keypair::StackKeyPair;
use dsnp_graph_config::{
	errors::{DsnpGraphError, DsnpGraphResult},
	ConnectionType, Environment, GraphKeyType, InputValidation, SchemaId,
};
use log::Level;
use log_result_proc_macro::log_result_err;
use std::{
	collections::{hash_map::Entry, HashSet},
	sync::{Arc, RwLock},
};

use super::api_types::GraphKeyPair;

/// Root data structure that stores all underlying data structures inside
#[derive(Debug)]
pub struct GraphState {
	/// Environment of this `GraphState`
	environment: Environment,

	/// Stores all shared states between different users, it can be used for PRId calculations or
	/// a repository for published public keys
	shared_state_manager: Arc<RwLock<SharedStateManager>>,

	/// Dsnp users and their corresponding social graphs
	user_map: TransactionalHashMap<DsnpUserId, UserGraph>,
}

/// Defines the main API to interact with Graph
pub trait GraphAPI {
	/// Checks if graph state contains a user
	fn contains_user_graph(&self, user_id: &DsnpUserId) -> bool;

	/// Returns number of users in the current graph state
	fn len(&self) -> usize;

	/// Removes the user graph from an SDK instance
	fn remove_user_graph(&mut self, user_id: &DsnpUserId);

	/// Imports raw data retrieved from the blockchain into users graph.
	/// Will overwrite any existing graph data for any existing user,
	/// but pending updates will be preserved.
	fn import_users_data(&mut self, payloads: &Vec<ImportBundle>) -> DsnpGraphResult<()>;

	/// Calculates the necessary new key and graph page updates for all imported users and graph using their active
	/// encryption key and return a list of updates
	fn export_updates(&self) -> DsnpGraphResult<Vec<Update>>;

	/// Calculates the necessary graph page updates for a single user, using their active encryption
	/// key, and returns a list of graph page updates
	fn export_user_graph_updates(&self, user_id: &DsnpUserId) -> DsnpGraphResult<Vec<Update>>;

	/// Applies Actions (Connect or Disconnect) to the list of pending actions for a users graph
	fn apply_actions(
		&mut self,
		action: &[Action],
		options: &Option<ActionOptions>,
	) -> DsnpGraphResult<()>;

	/// Force re-calculates the imported graphs. This is useful to ensure the pages are using the
	/// latest encryption key or refresh calculated PRIds or remove any empty pages and ...
	fn force_recalculate_graphs(&self, user_id: &DsnpUserId) -> DsnpGraphResult<Vec<Update>>;

	/// Gets a list of all connections of the indicated type for the user
	fn get_connections_for_user_graph(
		&self,
		user_id: &DsnpUserId,
		schema_id: &SchemaId,
		include_pending: bool,
	) -> DsnpGraphResult<Vec<DsnpGraphEdge>>;

	/// returns a list dsnp user ids that require keys
	fn get_connections_without_keys(&self) -> DsnpGraphResult<Vec<DsnpUserId>>;

	/// Gets a list of all private friendship connections that are only valid from users side
	fn get_one_sided_private_friendship_connections(
		&self,
		user_id: &DsnpUserId,
	) -> DsnpGraphResult<Vec<DsnpGraphEdge>>;

	/// Gets a list published and imported public keys associated with a user
	fn get_public_keys(&self, user_id: &DsnpUserId) -> DsnpGraphResult<Vec<DsnpPublicKey>>;

	/// Returns the deserialized dsnp keys without importing
	fn deserialize_dsnp_keys(keys: &Option<DsnpKeys>) -> DsnpGraphResult<Vec<DsnpPublicKey>>;

	/// Generate a key pair for the given key pair type
	fn generate_keypair(key_pair_type: GraphKeyType) -> DsnpGraphResult<GraphKeyPair>;
}

/// Provides transactional operation support on `GraphState`
impl Transactional for GraphState {
	/// Commits all underlying changes
	fn commit(&mut self) {
		let ids: Vec<_> = self.user_map.inner().keys().copied().collect();
		for uid in ids {
			if let Some(u) = self.user_map.get_mut(&uid) {
				u.commit();
			}
		}
		self.user_map.commit();
		self.shared_state_manager.write().unwrap().commit();
	}

	/// Rollbacks all underlying changes
	fn rollback(&mut self) {
		self.user_map.rollback();
		let ids: Vec<_> = self.user_map.inner().keys().copied().collect();
		for uid in ids {
			if let Some(u) = self.user_map.get_mut(&uid) {
				u.rollback();
			}
		}
		self.shared_state_manager.write().unwrap().rollback();
	}
}

/// Implementing GraphAPI functionalities on GraphState
impl GraphAPI for GraphState {
	/// Checks if graph state contains a user
	fn contains_user_graph(&self, user_id: &DsnpUserId) -> bool {
		self.user_map.inner().contains_key(user_id)
	}

	/// Returns number of users in the current graph state
	fn len(&self) -> usize {
		self.user_map.len()
	}

	/// Removes the user graph from an instance
	fn remove_user_graph(&mut self, user_id: &DsnpUserId) {
		self.user_map.remove(user_id);
		self.user_map.commit();
	}

	/// Imports raw data retrieved from the blockchain into a user graph.
	/// Will overwrite any existing graph data for the user,
	/// but pending updates will be preserved.
	#[log_result_err(Level::Error)]
	fn import_users_data(&mut self, payloads: &Vec<ImportBundle>) -> DsnpGraphResult<()> {
		let result = self.do_import_users_data(payloads);
		match result {
			DsnpGraphResult::Ok(_) => self.commit(),
			DsnpGraphResult::Err(_) => self.rollback(),
		};
		result
	}

	/// Calculates the necessary page updates for all users graphs and return as a map of pages to
	/// be updated and/or removed or added keys
	#[log_result_err(Level::Error)]
	fn export_updates(&self) -> DsnpGraphResult<Vec<Update>> {
		let mut result = self
			.shared_state_manager
			.read()
			.map_err(|_| DsnpGraphError::FailedtoReadLock(SHARED_STATE_MANAGER.to_string()))?
			.export_new_key_updates()?;
		let imported_users: Vec<_> = self.user_map.inner().keys().copied().collect();
		for user_id in imported_users {
			let user_graph = self
				.user_map
				.get(&user_id)
				.ok_or(DsnpGraphError::UserGraphNotImported(user_id))?;
			let updates = user_graph.calculate_updates()?;
			result.extend(updates);
		}
		Ok(result)
	}

	/// Calculates the necessary page updates for all users graphs and return as a map of pages to
	/// be updated and/or removed or added keys
	#[log_result_err(Level::Error)]
	fn export_user_graph_updates(&self, user_id: &DsnpUserId) -> DsnpGraphResult<Vec<Update>> {
		let mut result = self
			.shared_state_manager
			.read()
			.map_err(|_| DsnpGraphError::FailedtoReadLock(SHARED_STATE_MANAGER.to_string()))?
			.export_new_key_updates_for_user(user_id)?;
		let user_graph = self
			.user_map
			.get(&user_id)
			.ok_or(DsnpGraphError::UserGraphNotImported(*user_id))?;
		let updates = user_graph.calculate_updates()?;
		result.extend(updates);
		Ok(result)
	}

	/// Applies actions (Connect, Disconnect) to imported users graph
	#[log_result_err(Level::Error)]
	fn apply_actions(
		&mut self,
		actions: &[Action],
		options: &Option<ActionOptions>,
	) -> DsnpGraphResult<()> {
		let disable_auto_commit = match options {
			Some(ActionOptions { disable_auto_commit, .. }) => disable_auto_commit,
			None => &false,
		};

		let result = self.do_apply_actions(actions, options);

		if !disable_auto_commit {
			match result {
				DsnpGraphResult::Ok(_) => self.commit(),
				DsnpGraphResult::Err(_) => self.rollback(),
			}
		}
		result
	}

	/// Exports the graph pages for a certain user encrypted using the latest published key
	#[log_result_err(Level::Error)]
	fn force_recalculate_graphs(&self, user_id: &DsnpUserId) -> DsnpGraphResult<Vec<Update>> {
		let user_graph = self
			.user_map
			.get(&user_id)
			.ok_or(DsnpGraphError::UserGraphNotImported(*user_id))?;

		user_graph.force_calculate_graphs()
	}

	/// Gets a list of all connections of the indicated type for the user
	#[log_result_err(Level::Error)]
	fn get_connections_for_user_graph(
		&self,
		user_id: &DsnpUserId,
		schema_id: &SchemaId,
		include_pending: bool,
	) -> DsnpGraphResult<Vec<DsnpGraphEdge>> {
		let user_graph = self
			.user_map
			.get(user_id)
			.ok_or(DsnpGraphError::UserGraphNotImported(*user_id))?;

		Ok(user_graph.get_all_connections_of(*schema_id, include_pending))
	}

	/// returns a list dsnp user ids that require keys
	#[log_result_err(Level::Error)]
	fn get_connections_without_keys(&self) -> DsnpGraphResult<Vec<DsnpUserId>> {
		let private_friendship_schema_id = self
			.environment
			.get_config()
			.get_schema_id_from_connection_type(ConnectionType::Friendship(PrivacyType::Private))
			.ok_or(DsnpGraphError::InvalidPrivateSchemaId)?;
		let all_connections: HashSet<_> = self
			.user_map
			.inner()
			.values()
			.flat_map(|user_graph| {
				user_graph.get_all_connections_of(private_friendship_schema_id, true)
			})
			.map(|edge| edge.user_id)
			.collect();
		Ok(self
			.shared_state_manager
			.read()
			.map_err(|_| DsnpGraphError::FailedtoReadLock(SHARED_STATE_MANAGER.to_string()))?
			.find_users_without_keys(all_connections.into_iter().collect()))
	}

	/// Gets a list of all private friendship connections that are only valid from users side
	#[log_result_err(Level::Error)]
	fn get_one_sided_private_friendship_connections(
		&self,
		user_id: &DsnpUserId,
	) -> DsnpGraphResult<Vec<DsnpGraphEdge>> {
		let private_friendship_schema_id = self
			.environment
			.get_config()
			.get_schema_id_from_connection_type(ConnectionType::Friendship(PrivacyType::Private))
			.ok_or(DsnpGraphError::InvalidPrivateSchemaId)?;
		let user_graph = match self.user_map.get(user_id) {
			Some(graph) => graph,
			None => return Err(DsnpGraphError::UserGraphNotImported(*user_id)),
		};
		let graph = user_graph
			.graph(&private_friendship_schema_id)
			.ok_or(DsnpGraphError::InvalidSchemaId(private_friendship_schema_id))?;
		graph.get_one_sided_friendships()
	}

	/// Gets a list published and imported public keys associated with a user
	fn get_public_keys(&self, user_id: &DsnpUserId) -> DsnpGraphResult<Vec<DsnpPublicKey>> {
		Ok(self
			.shared_state_manager
			.read()
			.map_err(|_| DsnpGraphError::FailedtoReadLock(SHARED_STATE_MANAGER.to_string()))?
			.get_public_keys(user_id))
	}

	/// Returns the deserialized dsnp keys
	fn deserialize_dsnp_keys(keys: &Option<DsnpKeys>) -> DsnpGraphResult<Vec<DsnpPublicKey>> {
		// sorting by index in ascending mode
		let mut sorted_keys = match keys {
			Some(keys) => keys.keys.clone().to_vec(),
			None => vec![],
		};
		sorted_keys.sort();

		let mut dsnp_keys = vec![];
		for key in sorted_keys {
			let mut k =
				Frequency::read_public_key(&key.content).map_err(|e| DsnpGraphError::from(e))?;
			// key id is the itemized index of the key stored in Frequency
			k.key_id = Some(key.index.into());
			dsnp_keys.push(k);
		}
		Ok(dsnp_keys)
	}

	/// Generate a key pair for the given key pair type
	fn generate_keypair(key_pair_type: GraphKeyType) -> DsnpGraphResult<GraphKeyPair> {
		let key_pair = match key_pair_type {
			GraphKeyType::X25519 => StackKeyPair::gen(),
		};
		Ok(GraphKeyPair {
			secret_key: key_pair.secret_key.to_vec(),
			public_key: key_pair.public_key.to_vec(),
			key_type: key_pair_type,
		})
	}
}

/// inner functions for `GraphState`
impl GraphState {
	/// creates a new graph state with the given `Environment`
	pub fn new(environment: Environment) -> Self {
		Self {
			environment,
			user_map: TransactionalHashMap::new(),
			shared_state_manager: Arc::new(RwLock::new(SharedStateManager::new())),
		}
	}

	/// Gets an existing or creates a new UserGraph
	fn get_or_create_user_graph(
		&mut self,
		dsnp_user_id: DsnpUserId,
	) -> DsnpGraphResult<&mut UserGraph> {
		match self.user_map.entry(dsnp_user_id) {
			Entry::Occupied(o) => Ok(o.into_mut()),
			Entry::Vacant(v) => Ok(v.insert(UserGraph::new(
				&dsnp_user_id,
				&self.environment,
				self.shared_state_manager.clone(),
			))),
		}
	}

	/// main data importing logic
	#[log_result_err(Level::Error)]
	fn do_import_users_data(&mut self, payloads: &Vec<ImportBundle>) -> DsnpGraphResult<()> {
		for bundle in payloads {
			bundle.validate()?;
		}
		for ImportBundle { schema_id, pages, dsnp_keys, dsnp_user_id, key_pairs } in payloads {
			let connection_type_option =
				self.environment.get_config().get_connection_type_from_schema_id(*schema_id);

			match dsnp_keys {
				Some(dsnp_keys) => {
					self.shared_state_manager
						.write()
						.map_err(|_| {
							DsnpGraphError::FailedtoWriteLock(SHARED_STATE_MANAGER.to_string())
						})?
						.import_dsnp_keys(&dsnp_keys)?;
				},
				None => (),
			};
			let user_graph = self.get_or_create_user_graph(*dsnp_user_id)?;

			let include_secret_keys = !key_pairs.is_empty();
			{
				let mut user_key_manager = user_graph
					.user_key_manager
					.write()
					.map_err(|_| DsnpGraphError::FailedtoWriteLock(USER_KEY_MANAGER.to_string()))?;

				user_key_manager.import_key_pairs(key_pairs.clone())?;
			};

			if pages.is_empty() {
				// case where only keys are imported
				continue;
			}

			let dsnp_config = user_graph
				.get_dsnp_config(*schema_id)
				.ok_or(DsnpGraphError::InvalidSchemaId(*schema_id))?;

			let graph = user_graph
				.graph_mut(&schema_id)
				.ok_or(DsnpGraphError::InvalidSchemaId(*schema_id))?;
			graph.clear();

			let connection_type =
				connection_type_option.ok_or(DsnpGraphError::InvalidSchemaId(*schema_id))?;

			match connection_type.privacy_type() {
				PrivacyType::Public => {
					graph.import_public(connection_type, pages)?;
					user_graph.sync_updates(*schema_id);
				},
				PrivacyType::Private => {
					// private keys are provided try to import the graph
					if include_secret_keys {
						graph.import_private(&dsnp_config, connection_type, pages)?;
						user_graph.sync_updates(*schema_id);
					}

					// since it's a private friendship import provided PRIs
					if connection_type == ConnectionType::Friendship(PrivacyType::Private) {
						self.shared_state_manager
							.write()
							.map_err(|_| {
								DsnpGraphError::FailedtoWriteLock(SHARED_STATE_MANAGER.to_string())
							})?
							.import_pri(*dsnp_user_id, pages)?;
					}
				},
			};
		}
		Ok(())
	}

	/// main updating logic
	#[log_result_err(Level::Error)]
	fn do_apply_actions(
		&mut self,
		actions: &[Action],
		options: &Option<ActionOptions>,
	) -> DsnpGraphResult<()> {
		// pre validate all actions
		for action in actions {
			action.validate()?;
		}

		let (ignore_existing_connections, ignore_missing_connections) = match options {
			Some(options) =>
				(options.ignore_existing_connections, options.ignore_missing_connections),
			None => (false, false),
		};
		// apply actions
		for action in actions {
			let owner_graph = self.get_or_create_user_graph(action.owner_dsnp_user_id())?;
			match action {
				Action::Connect {
					connection: Connection { ref dsnp_user_id, ref schema_id },
					dsnp_keys,
					..
				} => {
					if owner_graph.graph_has_connection(*schema_id, *dsnp_user_id, true) {
						if ignore_existing_connections {
							log::warn!(
								"Ignoring add redundant connection {} -> {}",
								action.owner_dsnp_user_id(),
								*dsnp_user_id
							);
							continue;
						}

						return Err(DsnpGraphError::ConnectionAlreadyExists(
							action.owner_dsnp_user_id(),
							*dsnp_user_id,
						));
					}
					owner_graph.update_tracker_mut().register_update(
						UpdateEvent::create_add(*dsnp_user_id, *schema_id),
						ignore_existing_connections,
					)?;
					if let Some(inner_keys) = dsnp_keys {
						self.shared_state_manager
							.write()
							.map_err(|_| {
								DsnpGraphError::FailedtoWriteLock(SHARED_STATE_MANAGER.to_string())
							})?
							.import_dsnp_keys(inner_keys)?;
					}
				},
				Action::Disconnect {
					connection: Connection { ref dsnp_user_id, ref schema_id },
					..
				} => {
					if !owner_graph.graph_has_connection(*schema_id, *dsnp_user_id, true) {
						if ignore_missing_connections {
							log::warn!(
								"Ignoring remove non-existent connection {} -> {}",
								action.owner_dsnp_user_id(),
								*dsnp_user_id
							);
							continue;
						}

						return Err(DsnpGraphError::ConnectionDoesNotExist(
							action.owner_dsnp_user_id(),
							*dsnp_user_id,
						));
					}
					owner_graph.update_tracker_mut().register_update(
						UpdateEvent::create_remove(*dsnp_user_id, *schema_id),
						ignore_missing_connections,
					)?;
				},
				Action::AddGraphKey { new_public_key, .. } => {
					self.shared_state_manager
						.write()
						.map_err(|_| {
							DsnpGraphError::FailedtoWriteLock(SHARED_STATE_MANAGER.to_string())
						})?
						.add_new_key(action.owner_dsnp_user_id(), new_public_key.clone())?;
				},
			}
		}
		Ok(())
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::{
		api::api_types::ResolvedKeyPair,
		dsnp::{dsnp_configs::KeyPairType, dsnp_types::DsnpPrid},
		util::builders::{ImportBundleBuilder, KeyDataBuilder},
	};
	use memory_stats::memory_stats;
	use ntest::*;

	#[test]
	fn graph_contains_false() {
		let state = GraphState::new(Environment::Mainnet);
		assert!(!state.contains_user_graph(&0));
	}

	#[test]
	fn graph_contains_true() {
		let mut state = GraphState::new(Environment::Mainnet);
		let _ = state.get_or_create_user_graph(0);
		assert!(state.contains_user_graph(&0));
	}

	#[test]
	fn graph_len() {
		let mut state = GraphState::new(Environment::Mainnet);
		let _ = state.get_or_create_user_graph(0);
		assert_eq!(state.len(), 1);
		let _ = state.get_or_create_user_graph(1);
		assert_eq!(state.len(), 2);
	}

	#[test]
	fn add_user_success() {
		let mut state = GraphState::new(Environment::Mainnet);
		let res = state.get_or_create_user_graph(0);
		assert!(res.is_ok());
	}

	#[test]
	fn remove_user_success() {
		let mut state = GraphState::new(Environment::Mainnet);
		let _ = state.get_or_create_user_graph(0);
		let _ = state.get_or_create_user_graph(1);
		state.remove_user_graph(&0);
		assert_eq!(state.len(), 1);
		assert!(!state.contains_user_graph(&0));
		assert!(state.contains_user_graph(&1));
	}

	#[test]
	fn remove_nonexistent_user_noop() {
		let mut state = GraphState::new(Environment::Mainnet);
		let _ = state.get_or_create_user_graph(0);
		let _ = state.get_or_create_user_graph(1);
		state.remove_user_graph(&99);
		assert_eq!(state.user_map.len(), 2);
	}

	#[test]
	fn import_user_data_should_import_keys_and_data_for_public_follow_graph() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(ConnectionType::Follow(PrivacyType::Public))
			.expect("should exist");
		let mut state = GraphState::new(env.clone());
		let key_pair_raw = StackKeyPair::gen();
		let keypair = GraphKeyPair {
			secret_key: key_pair_raw.secret_key.to_vec(),
			public_key: key_pair_raw.public_key.to_vec(),
			key_type: GraphKeyType::X25519,
		};
		let dsnp_user_id = 123;
		let connections = vec![(2, 0), (3, 0), (4, 0), (5, 0)];
		let input = ImportBundleBuilder::new(env, dsnp_user_id, schema_id)
			.with_key_pairs(&vec![keypair.clone()])
			.with_page(1, &connections, &vec![], 1000)
			.build();

		// act
		let res = state.import_users_data(&vec![input]);

		// assert
		assert!(res.is_ok());

		let public_manager = state.shared_state_manager.read().unwrap();
		let keys = public_manager.get_imported_keys(dsnp_user_id);
		assert_eq!(keys.len(), 1);

		let res = state.get_connections_for_user_graph(&dsnp_user_id, &schema_id, false);
		assert!(res.is_ok());
		let res_set: HashSet<_> = res.unwrap().iter().copied().collect();
		let mapped: HashSet<_> = connections
			.into_iter()
			.map(|(c, s)| DsnpGraphEdge { user_id: c, since: s })
			.collect();
		assert_eq!(res_set, mapped);
	}

	#[test]
	fn import_user_data_should_import_keys_and_data_for_private_follow_graph() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(ConnectionType::Follow(PrivacyType::Private))
			.expect("should exist");
		let mut state = GraphState::new(env.clone());
		let key_pair_raw = StackKeyPair::gen();
		let resolved_key =
			ResolvedKeyPair { key_pair: KeyPairType::Version1_0(key_pair_raw.clone()), key_id: 1 };
		let keypair = GraphKeyPair {
			secret_key: key_pair_raw.secret_key.to_vec(),
			public_key: key_pair_raw.public_key.to_vec(),
			key_type: GraphKeyType::X25519,
		};
		let dsnp_user_id = 123;
		let connections = vec![(2, 0), (3, 0), (4, 0), (5, 0)];
		let input = ImportBundleBuilder::new(env, dsnp_user_id, schema_id)
			.with_key_pairs(&vec![keypair])
			.with_encryption_key(resolved_key)
			.with_page(1, &connections, &vec![], 100)
			.build();

		// act
		let res = state.import_users_data(&vec![input]);

		// assert
		assert!(res.is_ok());

		let public_manager = state.shared_state_manager.read().unwrap();
		let keys = public_manager.get_imported_keys(dsnp_user_id);
		assert_eq!(keys.len(), 1);

		let res = state.get_connections_for_user_graph(&dsnp_user_id, &schema_id, false);
		assert!(res.is_ok());
		let res_set: HashSet<_> = res.unwrap().iter().copied().collect();
		let mapped: HashSet<_> = connections
			.into_iter()
			.map(|(c, s)| DsnpGraphEdge { user_id: c, since: s })
			.collect();
		assert_eq!(res_set, mapped);
	}

	#[test]
	#[timeout(100000)]
	fn add_large_number_of_follows_to_private_follow_graph_should_succeed() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(ConnectionType::Follow(PrivacyType::Private))
			.expect("should exist");
		let mut state = GraphState::new(env.clone());
		let key_pair_raw = StackKeyPair::gen();
		let resolved_key =
			ResolvedKeyPair { key_pair: KeyPairType::Version1_0(key_pair_raw.clone()), key_id: 1 };
		let keypair = GraphKeyPair {
			secret_key: key_pair_raw.secret_key.to_vec(),
			public_key: key_pair_raw.public_key.to_vec(),
			key_type: GraphKeyType::X25519,
		};
		let dsnp_user_id = 7002;
		let input = ImportBundleBuilder::new(env.clone(), dsnp_user_id, schema_id)
			.with_key_pairs(&vec![keypair.clone()])
			.with_encryption_key(resolved_key.clone())
			.build();

		// act
		let mem_usage = memory_stats().unwrap();
		println!("before data import physical mem: {}", mem_usage.physical_mem);

		let res = state.import_users_data(&vec![input]);

		let mem_usage = memory_stats().unwrap();
		println!("after data import physical mem: {}", mem_usage.physical_mem);

		// assert
		assert!(res.is_ok());

		let actions: Vec<Action> = (1u64..7000u64)
			.map(|id| Action::Connect {
				owner_dsnp_user_id: dsnp_user_id,
				connection: Connection { dsnp_user_id: id, schema_id },
				dsnp_keys: None,
			})
			.collect();
		let mem_usage = memory_stats().unwrap();
		println!("before action import physical mem: {}", mem_usage.physical_mem);

		let res = state.apply_actions(
			&actions,
			&Some(ActionOptions {
				ignore_existing_connections: true,
				ignore_missing_connections: false,
				disable_auto_commit: false,
			}),
		);

		let mem_usage = memory_stats().unwrap();
		println!("after action import physical mem: {}", mem_usage.physical_mem);

		// assert
		assert!(res.is_ok());

		let connections =
			state.get_connections_for_user_graph(&dsnp_user_id, &schema_id, true).unwrap();
		let before_export_set: HashSet<_> = connections.iter().map(|e| e.user_id).collect();

		let export = state.export_updates();

		assert!(export.is_ok());
		println!("after export physical mem: {}", mem_usage.physical_mem);

		let updates = export.unwrap();

		let mut updated_state = GraphState::new(env.clone());
		let updated_input = ImportBundleBuilder::new(env.clone(), dsnp_user_id, schema_id)
			.with_key_pairs(&vec![keypair])
			.with_encryption_key(resolved_key.clone())
			.build();

		let new_import = ImportBundleBuilder::build_from(&updated_input, &updates);
		let res = updated_state.import_users_data(&vec![new_import]);

		assert!(res.is_ok());

		let connections = updated_state
			.get_connections_for_user_graph(&dsnp_user_id, &schema_id, false)
			.unwrap();
		let after_reimport_set: HashSet<_> = connections.iter().map(|e| e.user_id).collect();
		assert_eq!(before_export_set, after_reimport_set);
	}

	#[test]
	fn import_user_data_without_private_keys_should_add_prids_for_private_friendship_graph() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(ConnectionType::Friendship(PrivacyType::Private))
			.expect("should exist");
		let mut state = GraphState::new(env.clone());
		let dsnp_user_id = 123;
		let connections = vec![(2, 0), (3, 0), (4, 0), (5, 0)];
		let prids = vec![
			DsnpPrid::new(&[1, 2, 3, 4, 5, 6, 7, 4]),
			DsnpPrid::new(&[10, 2, 3, 4, 5, 6, 7, 4]),
			DsnpPrid::new(&[8, 2, 0, 4, 5, 6, 7, 4]),
			DsnpPrid::new(&[3, 2, 3, 4, 4, 6, 1, 4]),
		];
		let input = ImportBundleBuilder::new(env, dsnp_user_id, schema_id)
			.with_page(1, &connections, &prids, 1000)
			.build();

		// act
		let res = state.import_users_data(&vec![input]);

		// assert
		assert!(res.is_ok());

		let manager = state.shared_state_manager.read().unwrap();
		for p in prids {
			assert!(manager.contains(dsnp_user_id, p));
		}
	}

	#[test]
	fn import_user_data_with_wrong_key_should_fail_for_private_follow_graph_and_rollback_everything(
	) {
		// arrange
		let env = Environment::Mainnet;
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(ConnectionType::Follow(PrivacyType::Private))
			.expect("should exist");
		let mut state = GraphState::new(env.clone());
		let key_pair_raw = StackKeyPair::gen();
		let resolved_key =
			ResolvedKeyPair { key_pair: KeyPairType::Version1_0(key_pair_raw.clone()), key_id: 1 };
		let keypair = GraphKeyPair {
			secret_key: key_pair_raw.secret_key.to_vec(),
			public_key: key_pair_raw.public_key.to_vec(),
			key_type: GraphKeyType::X25519,
		};
		let dsnp_user_id = 123;
		let connections = vec![(2, 0), (3, 0), (4, 0), (5, 0)];
		let mut input = ImportBundleBuilder::new(env, dsnp_user_id, schema_id)
			.with_key_pairs(&vec![keypair])
			.with_encryption_key(resolved_key)
			.with_page(1, &connections, &vec![], 0)
			.build();
		let wrong_key_pair = StackKeyPair::gen();
		input.key_pairs = vec![GraphKeyPair {
			secret_key: wrong_key_pair.secret_key.to_vec(),
			public_key: wrong_key_pair.public_key.to_vec(),
			key_type: GraphKeyType::X25519,
		}];

		// act
		let res = state.import_users_data(&vec![input]);

		// assert
		assert!(res.is_err());
		assert_eq!(
			state.shared_state_manager.read().unwrap().get_imported_keys(dsnp_user_id).len(),
			0
		);
		assert!(state.get_connections_for_user_graph(&dsnp_user_id, &schema_id, true).is_err());
	}

	#[test]
	fn apply_actions_error_should_rollback_every_action() {
		let env = Environment::Mainnet;
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(ConnectionType::Follow(PrivacyType::Private))
			.expect("should exist");
		let key_pair_raw = StackKeyPair::gen();
		let keypair = GraphKeyPair {
			secret_key: key_pair_raw.secret_key.to_vec(),
			public_key: key_pair_raw.public_key.to_vec(),
			key_type: GraphKeyType::X25519,
		};
		let owner_dsnp_user_id: DsnpUserId = 0;
		let connect_action_1 = Action::Connect {
			owner_dsnp_user_id,
			connection: Connection { dsnp_user_id: 1, schema_id },
			dsnp_keys: Some(DsnpKeys {
				keys: KeyDataBuilder::new().with_key_pairs(&vec![keypair]).build(),
				keys_hash: 0,
				dsnp_user_id: owner_dsnp_user_id,
			}),
		};
		let connect_action_2 = Action::Connect {
			owner_dsnp_user_id,
			connection: Connection { dsnp_user_id: 2, schema_id },
			dsnp_keys: None,
		};

		let key_add_action = Action::AddGraphKey {
			owner_dsnp_user_id,
			new_public_key: b"27893788291911998228288282".to_vec(),
		};
		let mut state = GraphState::new(env);

		// act
		assert!(state
			.apply_actions(
				&vec![connect_action_1.clone(), connect_action_2, connect_action_1, key_add_action],
				&None
			)
			.is_err());

		// assert
		assert_eq!(state.user_map.len(), 0);
		let updates = state.shared_state_manager.write().unwrap().export_new_key_updates();
		assert!(updates.is_ok());
		assert_eq!(updates.unwrap().len(), 0);
	}
}
