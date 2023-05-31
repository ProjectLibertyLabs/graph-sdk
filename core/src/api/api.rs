use crate::{
	dsnp::{
		api_types::{Action, Connection, ImportBundle, PrivacyType, Update},
		dsnp_types::{DsnpGraphEdge, DsnpPublicKey, DsnpUserId},
	},
	graph::{
		key_manager::UserKeyProvider,
		shared_state_manager::{PriProvider, PublicKeyProvider, SharedStateManager},
		updates::UpdateEvent,
		user::UserGraph,
	},
	util::transactional_hashmap::{Transactional, TransactionalHashMap},
};
use dsnp_graph_config::{
	errors::{DsnpGraphError, DsnpGraphResult},
	ConnectionType, Environment, InputValidation, SchemaId,
};
use std::{
	cmp::min,
	collections::{hash_map::Entry, HashSet},
	sync::{Arc, RwLock},
};

#[derive(Debug)]
pub struct GraphState {
	capacity: usize,
	environment: Environment,
	shared_state_manager: Arc<RwLock<SharedStateManager>>,
	user_map: TransactionalHashMap<DsnpUserId, UserGraph>,
}

pub trait GraphAPI {
	/// Check if graph state contains a user
	fn contains_user_graph(&self, user_id: &DsnpUserId) -> bool;

	/// Return number of users in the current graph state
	fn len(&self) -> usize;

	/// Remove the user graph from an SDK instance
	fn remove_user_graph(&mut self, user_id: &DsnpUserId);

	/// Import raw data retrieved from the blockchain into users graph.
	/// Will overwrite any existing graph data for any existing user,
	/// but pending updates will be preserved.
	fn import_users_data(&mut self, payloads: &Vec<ImportBundle>) -> DsnpGraphResult<()>;

	/// Calculate the necessary page updates for all imported users and graph using their active
	/// encryption key and return a list of updates
	fn export_updates(&self) -> DsnpGraphResult<Vec<Update>>;

	/// Apply Actions (Connect or Disconnect) to the list of pending actions for a users graph
	fn apply_actions(&mut self, action: &[Action]) -> DsnpGraphResult<()>;

	/// Force re-calculates the imported graphs. This is useful to ensure the pages are using the
	/// latest encryption key or refresh calculated PRIds or remove any empty pages and ...
	fn force_recalculate_graphs(&self, user_id: &DsnpUserId) -> DsnpGraphResult<Vec<Update>>;

	/// Get a list of all connections of the indicated type for the user
	fn get_connections_for_user_graph(
		&self,
		user_id: &DsnpUserId,
		schema_id: &SchemaId,
		include_pending: bool,
	) -> DsnpGraphResult<Vec<DsnpGraphEdge>>;

	/// return a list dsnp user ids that require keys
	fn get_connections_without_keys(&self) -> DsnpGraphResult<Vec<DsnpUserId>>;

	/// Get a list of all private friendship connections that are only valid from users side
	fn get_one_sided_private_friendship_connections(
		&self,
		user_id: &DsnpUserId,
	) -> DsnpGraphResult<Vec<DsnpGraphEdge>>;

	/// Get a list published and imported public keys associated with a user
	fn get_public_keys(&self, user_id: &DsnpUserId) -> DsnpGraphResult<Vec<DsnpPublicKey>>;
}

impl Transactional for GraphState {
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

impl GraphAPI for GraphState {
	/// Check if graph state contains a user
	fn contains_user_graph(&self, user_id: &DsnpUserId) -> bool {
		self.user_map.inner().contains_key(user_id)
	}

	/// Return number of users in the current graph state
	fn len(&self) -> usize {
		self.user_map.len()
	}

	/// Remove the user graph from an instance
	fn remove_user_graph(&mut self, user_id: &DsnpUserId) {
		self.user_map.remove(user_id);
		self.user_map.commit();
	}

	/// Import raw data retrieved from the blockchain into a user graph.
	/// Will overwrite any existing graph data for the user,
	/// but pending updates will be preserved.
	fn import_users_data(&mut self, payloads: &Vec<ImportBundle>) -> DsnpGraphResult<()> {
		let result = self.do_import_users_data(payloads);
		match result {
			DsnpGraphResult::Ok(_) => self.commit(),
			DsnpGraphResult::Err(_) => self.rollback(),
		};
		result
	}

	/// Calculate the necessary page updates for all users graphs and return as a map of pages to
	/// be updated and/or removed or added keys
	fn export_updates(&self) -> DsnpGraphResult<Vec<Update>> {
		let mut result = self.shared_state_manager.read().unwrap().export_new_key_updates()?;
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

	/// Apply actions (Connect, Disconnect) to imported users graph
	fn apply_actions(&mut self, actions: &[Action]) -> DsnpGraphResult<()> {
		let result = self.do_apply_actions(actions);
		match result {
			DsnpGraphResult::Ok(_) => self.commit(),
			DsnpGraphResult::Err(_) => self.rollback(),
		};
		result
	}

	/// Export the graph pages for a certain user encrypted using the latest published key
	fn force_recalculate_graphs(&self, user_id: &DsnpUserId) -> DsnpGraphResult<Vec<Update>> {
		let user_graph = self
			.user_map
			.get(&user_id)
			.ok_or(DsnpGraphError::UserGraphNotImported(*user_id))?;

		user_graph.force_calculate_graphs()
	}

	/// Get a list of all connections of the indicated type for the user
	fn get_connections_for_user_graph(
		&self,
		user_id: &DsnpUserId,
		schema_id: &SchemaId,
		include_pending: bool,
	) -> DsnpGraphResult<Vec<DsnpGraphEdge>> {
		let user_graph = match self.user_map.get(user_id) {
			Some(graph) => graph,
			None => return Err(DsnpGraphError::UserGraphNotImported(*user_id)),
		};

		Ok(user_graph.get_all_connections_of(*schema_id, include_pending))
	}

	/// return a list dsnp user ids that require keys
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
			.unwrap()
			.find_users_without_keys(all_connections.into_iter().collect()))
	}

	/// Get a list of all private friendship connections that are only valid from users side
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

	/// Get a list published and imported public keys associated with a user
	fn get_public_keys(&self, user_id: &DsnpUserId) -> DsnpGraphResult<Vec<DsnpPublicKey>> {
		Ok(self
			.shared_state_manager
			.read()
			.map_err(|_| DsnpGraphError::FailedtoReadLockStateManager)?
			.get_public_keys(user_id))
	}
}

impl GraphState {
	pub fn new(environment: Environment) -> Self {
		Self {
			capacity: environment.get_config().sdk_max_users_graph_size as usize,
			environment,
			user_map: TransactionalHashMap::new(),
			shared_state_manager: Arc::new(RwLock::new(SharedStateManager::new())),
		}
	}

	pub fn with_capacity(environment: Environment, capacity: usize) -> Self {
		let size = min(capacity, environment.get_config().sdk_max_users_graph_size as usize);
		Self {
			environment,
			capacity: size,
			user_map: TransactionalHashMap::with_capacity(size),
			shared_state_manager: Arc::new(RwLock::new(SharedStateManager::new())),
		}
	}

	pub fn capacity(&self) -> usize {
		self.capacity
	}

	/// Gets an existing or creates a new UserGraph
	fn get_or_create_user_graph(
		&mut self,
		dsnp_user_id: DsnpUserId,
	) -> DsnpGraphResult<&mut UserGraph> {
		let is_full = self.user_map.len() >= self.capacity;
		match self.user_map.entry(dsnp_user_id) {
			Entry::Occupied(o) => Ok(o.into_mut()),
			Entry::Vacant(v) => {
				if is_full {
					return Err(DsnpGraphError::GraphStateIsFull)
				}
				Ok(v.insert(UserGraph::new(
					&dsnp_user_id,
					&self.environment,
					self.shared_state_manager.clone(),
				)))
			},
		}
	}

	fn do_import_users_data(&mut self, payloads: &Vec<ImportBundle>) -> DsnpGraphResult<()> {
		// pre validate all bundles
		for bundle in payloads {
			bundle.validate()?;
		}
		for ImportBundle { schema_id, pages, dsnp_keys, dsnp_user_id, key_pairs } in payloads {
			let connection_type = self
				.environment
				.get_config()
				.get_connection_type_from_schema_id(*schema_id)
				.ok_or(DsnpGraphError::InvalidSchemaId(*schema_id))?;
			self.shared_state_manager.write().unwrap().import_dsnp_keys(&dsnp_keys)?;

			let user_graph = self.get_or_create_user_graph(*dsnp_user_id)?;
			let dsnp_config = user_graph
				.get_dsnp_config(*schema_id)
				.ok_or(DsnpGraphError::InvalidSchemaId(*schema_id))?;

			let include_secret_keys = !key_pairs.is_empty();
			{
				let mut user_key_manager = user_graph.user_key_manager.write().unwrap();

				// import key-pairs inside user key manager
				user_key_manager.import_key_pairs(key_pairs.clone())?;
			};

			let graph = user_graph
				.graph_mut(&schema_id)
				.ok_or(DsnpGraphError::InvalidSchemaId(*schema_id))?;
			graph.clear();

			match connection_type.privacy_type() {
				PrivacyType::Public => graph.import_public(connection_type, pages),
				PrivacyType::Private => {
					// private keys are provided try to import the graph
					if include_secret_keys {
						graph.import_private(&dsnp_config, connection_type, pages)?;
					}

					// since it's a private friendship import provided PRIs
					if connection_type == ConnectionType::Friendship(PrivacyType::Private) {
						self.shared_state_manager
							.write()
							.unwrap()
							.import_pri(*dsnp_user_id, pages)?;
					}

					Ok(())
				},
			}?;
		}
		Ok(())
	}

	fn do_apply_actions(&mut self, actions: &[Action]) -> DsnpGraphResult<()> {
		// pre validate all actions
		for action in actions {
			action.validate()?;
		}
		for action in actions {
			let owner_graph = self.get_or_create_user_graph(action.owner_dsnp_user_id())?;
			match action {
				Action::Connect {
					connection: Connection { ref dsnp_user_id, ref schema_id },
					dsnp_keys,
					..
				} => {
					if owner_graph.graph_has_connection(*schema_id, *dsnp_user_id, true) {
						return Err(DsnpGraphError::ConnectionAlreadyExists(
							action.owner_dsnp_user_id(),
							*dsnp_user_id,
						))
					}
					owner_graph
						.update_tracker_mut()
						.register_update(&UpdateEvent::create_add(*dsnp_user_id, *schema_id))?;
					if let Some(inner_keys) = dsnp_keys {
						self.shared_state_manager
							.write()
							.map_err(|_| DsnpGraphError::FailedtoWriteLockStateManager)?
							.import_dsnp_keys(inner_keys)?;
					}
				},
				Action::Disconnect {
					connection: Connection { ref dsnp_user_id, ref schema_id },
					..
				} => {
					if !owner_graph.graph_has_connection(*schema_id, *dsnp_user_id, true) {
						return Err(DsnpGraphError::ConnectionDoesNotExist(
							action.owner_dsnp_user_id(),
							*dsnp_user_id,
						))
					}
					owner_graph
						.update_tracker_mut()
						.register_update(&UpdateEvent::create_remove(*dsnp_user_id, *schema_id))?;
				},
				Action::AddGraphKey { new_public_key, .. } => {
					self.shared_state_manager
						.write()
						.unwrap()
						.add_new_key(action.owner_dsnp_user_id(), new_public_key.clone())?;
				},
			}
		}
		Ok(())
	}
}

#[cfg(test)]
mod test {
	use crate::{
		dsnp::{
			api_types::{Connection, DsnpKeys, GraphKeyPair, ResolvedKeyPair},
			dsnp_configs::KeyPairType,
			dsnp_types::DsnpPrid,
		},
		util::builders::{ImportBundleBuilder, KeyDataBuilder},
	};
	use dryoc::keypair::StackKeyPair;
	use dsnp_graph_config::{builder::ConfigBuilder, ConnectionType, GraphKeyType};

	use super::*;

	const TEST_CAPACITY: usize = 10;

	#[test]
	fn new_graph_state_with_capacity_sets_initial_hash_map_capacity() {
		let env = Environment::Dev(
			ConfigBuilder::new().with_sdk_max_users_graph_size(TEST_CAPACITY as u32).build(),
		);
		let capacity: usize = 5;
		let new_state = GraphState::with_capacity(env, capacity);
		assert!(new_state.user_map.inner().capacity() >= capacity);
		assert_eq!(new_state.capacity, capacity);
	}

	#[test]
	fn new_graph_state_with_capacity_caps_initial_hash_map_capacity() {
		let env = Environment::Dev(
			ConfigBuilder::new().with_sdk_max_users_graph_size(TEST_CAPACITY as u32).build(),
		);
		let new_state = GraphState::with_capacity(env, TEST_CAPACITY * 2);
		assert!(new_state.user_map.inner().capacity() >= TEST_CAPACITY);
	}

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
	fn add_user_errors_if_graph_state_full() {
		let env = Environment::Dev(ConfigBuilder::new().with_sdk_max_users_graph_size(1).build());
		let mut state = GraphState::new(env);
		let _ = state.get_or_create_user_graph(0);
		assert!(state.get_or_create_user_graph(1).is_err());
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
	fn import_user_data_should_without_private_keys_should_add_prids_for_private_friendship_graph()
	{
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
			.apply_actions(&vec![
				connect_action_1.clone(),
				connect_action_2,
				connect_action_1,
				key_add_action
			])
			.is_err());

		// assert
		assert_eq!(state.user_map.len(), 0);
		let updates = state.shared_state_manager.write().unwrap().export_new_key_updates();
		assert!(updates.is_ok());
		assert_eq!(updates.unwrap().len(), 0);
	}
}
