use crate::{
	dsnp::{
		api_types::{Action, Connection, ImportBundle, PrivacyType, Update},
		dsnp_configs::DsnpVersionConfig,
		dsnp_types::{DsnpGraphEdge, DsnpUserId},
	},
	graph::{
		key_manager::UserKeyProvider,
		shared_state_manager::{PriProvider, PublicKeyProvider, SharedStateManager},
		updates::UpdateEvent,
		user::UserGraph,
	},
};
use anyhow::{Error, Ok, Result};
use dsnp_graph_config::{ConnectionType, Environment, SchemaId};
use std::{
	cell::RefCell,
	cmp::min,
	collections::{hash_map::Entry, HashMap, HashSet},
	ops::{Deref, DerefMut},
	rc::Rc,
};

#[derive(Debug)]
pub struct GraphState {
	environment: Environment,
	shared_state_manager: Rc<RefCell<SharedStateManager>>,
	user_map: HashMap<DsnpUserId, UserGraph>,
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
	fn import_users_data(&mut self, payloads: Vec<ImportBundle>) -> Result<()>;

	/// Calculate the necessary page updates for all imported users and graph using their active
	/// encryption key and return a list of updates
	fn export_updates(&mut self) -> Result<Vec<Update>>;

	/// Apply Actions (Connect or Disconnect) to the list of pending actions for a users graph
	fn apply_actions(&mut self, action: &[Action]) -> Result<()>;

	/// Get a list of all connections of the indicated type for the user
	fn get_connections_for_user_graph(
		&self,
		user_id: &DsnpUserId,
		schema_id: &SchemaId,
		include_pending: bool,
	) -> Result<Vec<DsnpGraphEdge>>;

	/// return a list dsnp user ids that require keys
	fn get_connections_without_keys(&self) -> Result<Vec<DsnpUserId>>;

	/// Get a list of all private friendship connections that are only valid from users side
	fn get_one_sided_private_friendship_connections(
		&self,
		user_id: &DsnpUserId,
	) -> Result<Vec<DsnpGraphEdge>>;
}

impl GraphState {
	pub fn new(environment: Environment) -> Self {
		Self {
			environment,
			user_map: HashMap::<DsnpUserId, UserGraph>::new(),
			shared_state_manager: Rc::new(RefCell::from(SharedStateManager::new())),
		}
	}

	pub fn with_capacity(environment: Environment, capacity: usize) -> Self {
		let size = min(capacity, environment.get_config().sdk_max_users_graph_size as usize);
		Self {
			environment,
			user_map: HashMap::<DsnpUserId, UserGraph>::with_capacity(size),
			shared_state_manager: Rc::new(RefCell::from(SharedStateManager::new())),
		}
	}

	pub fn capacity(&self) -> usize {
		self.environment.get_config().sdk_max_users_graph_size as usize
	}
}

impl GraphAPI for GraphState {
	/// Check if graph state contains a user
	fn contains_user_graph(&self, user_id: &DsnpUserId) -> bool {
		self.user_map.contains_key(user_id)
	}

	/// Return number of users in the current graph state
	fn len(&self) -> usize {
		self.user_map.len()
	}

	/// Remove the user graph from an instance
	fn remove_user_graph(&mut self, user_id: &DsnpUserId) {
		self.user_map.remove(user_id);
	}

	/// Import raw data retrieved from the blockchain into a user graph.
	/// Will overwrite any existing graph data for the user,
	/// but pending updates will be preserved.
	// TODO: should make it transactional
	fn import_users_data(&mut self, payloads: Vec<ImportBundle>) -> Result<()> {
		for ImportBundle { schema_id, pages, dsnp_keys, dsnp_user_id, key_pairs } in payloads {
			let dsnp_config = self
				.get_dsnp_config(schema_id)
				.ok_or(Error::msg("Invalid schema id for environment!"))?;
			let config = self.environment.get_config();
			let connection_type = config
				.get_connection_type_from_schema_id(schema_id)
				.ok_or(Error::msg("Invalid schema id for environment!"))?;
			self.shared_state_manager
				.deref()
				.borrow_mut()
				.deref_mut()
				.import_dsnp_keys(&dsnp_keys)?;

			let user_graph = self.get_or_create_user_graph(dsnp_user_id)?;

			let include_secret_keys = !key_pairs.is_empty();
			{
				let mut user_key_manager = user_graph.user_key_manager.borrow_mut();

				// import key-pairs inside user key manager
				user_key_manager.deref_mut().import_key_pairs(key_pairs)?;
			};

			let graph = user_graph.graph_mut(&schema_id);
			graph.clear();

			match (connection_type.privacy_type(), include_secret_keys) {
				(PrivacyType::Public, _) => graph.import_public(connection_type, pages),
				(PrivacyType::Private, true) => {
					graph.import_private(&dsnp_config, connection_type, &pages)?;
					self.shared_state_manager
						.deref()
						.borrow_mut()
						.deref_mut()
						.import_pri(dsnp_user_id, &pages)
				},
				(PrivacyType::Private, false) => self
					.shared_state_manager
					.deref()
					.borrow_mut()
					.deref_mut()
					.import_pri(dsnp_user_id, &pages),
			}?;
		}

		Ok(())
	}

	/// Calculate the necessary page updates for all users graphs and return as a map of pages to
	/// be updated and/or removed
	// TODO: should make it transactional
	fn export_updates(&mut self) -> Result<Vec<Update>> {
		let mut result = vec![];
		let keys: Vec<_> = self.user_map.keys().copied().collect();
		for user_id in keys {
			let schemas = self
				.user_map
				.get(&user_id)
				.ok_or(Error::msg("User not found for graph export"))?
				.update_tracker()
				.get_updated_schema_ids();
			let related_dsnp_configs: HashSet<DsnpVersionConfig> =
				schemas.iter().filter_map(|s| self.get_dsnp_config(*s)).collect();

			// we are checking user existence on previous lines so we can unwrap safely here
			let user_graph = self.user_map.get_mut(&user_id).unwrap();
			for dsnp_config in related_dsnp_configs.iter() {
				let updates = user_graph.calculate_updates(dsnp_config)?;
				result.extend(updates);
			}
		}
		Ok(result)
	}

	/// Apply actions (Connect, Disconnect) to imported users graph
	// TODO: should become transactional
	fn apply_actions(&mut self, actions: &[Action]) -> Result<()> {
		for action in actions {
			let owner_graph = self.get_or_create_user_graph(action.owner_dsnp_user_id())?;
			let update_event = match action {
				Action::Connect {
					connection: Connection { ref dsnp_user_id, ref schema_id },
					..
				} => {
					if owner_graph.graph_has_connection(*schema_id, *dsnp_user_id, true) {
						return Err(Error::msg(format!(
							"Connection from {} to {} already exists!",
							action.owner_dsnp_user_id(),
							dsnp_user_id
						)))
					}
					UpdateEvent::create_add(*dsnp_user_id, *schema_id)
				},
				Action::Disconnect {
					connection: Connection { ref dsnp_user_id, ref schema_id },
					..
				} => {
					if !owner_graph.graph_has_connection(*schema_id, *dsnp_user_id, true) {
						return Err(Error::msg(format!(
							"Connection from {} to {} does not exists to be disconnected!",
							action.owner_dsnp_user_id(),
							dsnp_user_id
						)))
					}
					UpdateEvent::create_remove(*dsnp_user_id, *schema_id)
				},
			};

			owner_graph.update_tracker_mut().register_update(&update_event)?;
		}
		Ok(())
	}

	/// Get a list of all connections of the indicated type for the user
	fn get_connections_for_user_graph(
		&self,
		user_id: &DsnpUserId,
		schema_id: &SchemaId,
		include_pending: bool,
	) -> Result<Vec<DsnpGraphEdge>> {
		let user_graph = match self.user_map.get(user_id) {
			Some(graph) => graph,
			None => return Err(Error::msg("user not present in graph state")),
		};

		Ok(user_graph.get_all_connections_of(*schema_id, include_pending))
	}

	/// return a list dsnp user ids that require keys
	fn get_connections_without_keys(&self) -> Result<Vec<DsnpUserId>> {
		let private_friendship_schema_id = self
			.environment
			.get_config()
			.get_schema_id_from_connection_type(ConnectionType::Friendship(PrivacyType::Private))
			.ok_or(Error::msg("Schema id for private friendship does not exists!"))?;
		let all_connections: HashSet<_> = self
			.user_map
			.values()
			.flat_map(|user_graph| {
				user_graph.get_all_connections_of(private_friendship_schema_id, true)
			})
			.map(|edge| edge.user_id)
			.collect();
		Ok(self
			.shared_state_manager
			.deref()
			.borrow()
			.find_users_without_keys(all_connections.into_iter().collect()))
	}

	/// Get a list of all private friendship connections that are only valid from users side
	fn get_one_sided_private_friendship_connections(
		&self,
		user_id: &DsnpUserId,
	) -> Result<Vec<DsnpGraphEdge>> {
		let private_friendship_schema_id = self
			.environment
			.get_config()
			.get_schema_id_from_connection_type(ConnectionType::Friendship(PrivacyType::Private))
			.ok_or(Error::msg("Schema id for private friendship does not exists!"))?;
		let user_graph = match self.user_map.get(user_id) {
			Some(graph) => graph,
			None => return Err(Error::msg("user not present in graph state")),
		};
		let graph = user_graph.graph(&private_friendship_schema_id);
		graph.get_one_sided_friendships()
	}
}

impl GraphState {
	fn get_dsnp_config(&self, schema_id: SchemaId) -> Option<DsnpVersionConfig> {
		let config = self.environment.get_config();
		if let Some(dsnp_version) = config.get_dsnp_version_from_schema_id(schema_id) {
			return Some(DsnpVersionConfig::new(dsnp_version))
		}
		None
	}

	/// Gets an existing or creates a new UserGraph
	fn get_or_create_user_graph(&mut self, dsnp_user_id: DsnpUserId) -> Result<&mut UserGraph> {
		let is_full =
			self.user_map.len() >= self.environment.get_config().sdk_max_users_graph_size as usize;
		match self.user_map.entry(dsnp_user_id) {
			Entry::Occupied(o) => Ok(o.into_mut()),
			Entry::Vacant(v) => {
				if is_full {
					return Err(Error::msg("GraphState instance full"))
				}
				Ok(v.insert(UserGraph::new(
					&dsnp_user_id,
					&self.environment,
					self.shared_state_manager.clone(),
				)))
			},
		}
	}
}

#[cfg(test)]
mod test {
	use crate::{
		dsnp::{
			api_types::{Connection, GraphKeyPair, ResolvedKeyPair},
			dsnp_configs::KeyPairType,
			dsnp_types::DsnpPrid,
		},
		tests::helpers::ImportBundleBuilder,
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
		assert!(new_state.user_map.capacity() >= capacity);
	}

	#[test]
	fn new_graph_state_with_capacity_caps_initial_hash_map_capacity() {
		let env = Environment::Dev(
			ConfigBuilder::new().with_sdk_max_users_graph_size(TEST_CAPACITY as u32).build(),
		);
		let new_state = GraphState::with_capacity(env, TEST_CAPACITY * 2);
		assert!(new_state.user_map.capacity() >= TEST_CAPACITY);
	}

	#[test]
	fn graph_state_capacity() {
		let env = Environment::Dev(
			ConfigBuilder::new().with_sdk_max_users_graph_size(TEST_CAPACITY as u32).build(),
		);
		let state = GraphState::new(env);
		assert_eq!(state.capacity(), TEST_CAPACITY);
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
			.with_page(1, &connections, &vec![], 0)
			.build();

		// act
		let res = state.import_users_data(vec![input]);

		// assert
		assert!(res.is_ok());

		let public_manager = state.shared_state_manager.borrow();
		let keys = public_manager.get_all_keys(dsnp_user_id);
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
			.with_page(1, &connections, &vec![], 0)
			.build();

		// act
		let res = state.import_users_data(vec![input]);

		// assert
		assert!(res.is_ok());

		let public_manager = state.shared_state_manager.borrow();
		let keys = public_manager.get_all_keys(dsnp_user_id);
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
			.with_page(1, &connections, &prids, 0)
			.build();

		// act
		let res = state.import_users_data(vec![input]);

		// assert
		assert!(res.is_ok());

		let manager = state.shared_state_manager.borrow();
		for p in prids {
			assert!(manager.contains(dsnp_user_id, p));
		}
	}

	#[test]
	fn import_user_data_with_wrong_key_should_fail_for_private_follow_graph() {
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
		let res = state.import_users_data(vec![input]);

		// assert
		assert!(res.is_err());
	}

	#[test]
	#[ignore = "todo"]
	fn export_user_updates() {}

	#[test]
	fn add_duplicate_connection_for_user_errors() {
		let owner_dsnp_user_id: DsnpUserId = 0;
		let env = Environment::Mainnet;
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(ConnectionType::Follow(PrivacyType::Private))
			.expect("should exist");
		let action = Action::Connect {
			owner_dsnp_user_id,
			connection: Connection { schema_id, dsnp_user_id: 1 },
		};

		let mut state = GraphState::new(env);
		assert!(state.apply_actions(&vec![action.clone()]).is_ok());
		assert!(state.apply_actions(&vec![action]).is_err());
	}

	#[test]
	fn remove_connection_for_user_twice_errors() {
		let env = Environment::Mainnet;
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(ConnectionType::Follow(PrivacyType::Private))
			.expect("should exist");
		let owner_dsnp_user_id: DsnpUserId = 0;
		let connect_action = Action::Connect {
			owner_dsnp_user_id,
			connection: Connection { dsnp_user_id: 1, schema_id },
		};
		let disconnect_action = Action::Disconnect {
			owner_dsnp_user_id,
			connection: Connection { dsnp_user_id: 1, schema_id },
		};
		let mut state = GraphState::new(env);
		assert!(state.apply_actions(&vec![connect_action]).is_ok());

		// act
		assert!(state.apply_actions(&vec![disconnect_action.clone()]).is_ok());
		assert!(state.apply_actions(&vec![disconnect_action]).is_err());
	}

	#[test]
	fn remove_connection_from_nonexistent_user_errors() {
		let env = Environment::Mainnet;
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(ConnectionType::Follow(PrivacyType::Private))
			.expect("should exist");
		let mut state = GraphState::new(env);
		assert!(state
			.apply_actions(&vec![Action::Disconnect {
				owner_dsnp_user_id: 0,
				connection: Connection { dsnp_user_id: 1, schema_id }
			}])
			.is_err());
	}

	#[test]
	fn get_connections_for_user_graph_with_pending_should_include_updates() {
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
			.with_page(1, &connections, &vec![], 0)
			.build();
		state.import_users_data(vec![input]).expect("should work");
		let actions = vec![
			Action::Connect {
				connection: Connection { schema_id, dsnp_user_id: 1 },
				owner_dsnp_user_id: dsnp_user_id,
			},
			Action::Disconnect {
				connection: Connection { schema_id, dsnp_user_id: 3 },
				owner_dsnp_user_id: dsnp_user_id,
			},
		];
		let expected_connections = HashSet::from([2, 4, 5, 1]);

		// act
		let action1 = &actions[0..1];
		let action2 = &actions[1..2];
		let res1 = state.apply_actions(action1);
		let res2 = state.apply_actions(action2);

		// assert
		assert!(res1.is_ok());
		assert!(res2.is_ok());

		let connections_result =
			state.get_connections_for_user_graph(&dsnp_user_id, &schema_id, true);
		assert!(connections_result.is_ok());
		let mapped: HashSet<_> =
			connections_result.unwrap().into_iter().map(|c| c.user_id).collect();
		assert_eq!(mapped, expected_connections);
	}
}
