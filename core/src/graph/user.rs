#![allow(dead_code)]
use crate::{api::api_types::*, dsnp::dsnp_types::*, graph::updates::UpdateTracker};
use dsnp_graph_config::{
	errors::{DsnpGraphError, DsnpGraphResult},
	Environment, SchemaId,
};
use std::{
	collections::HashSet,
	sync::{Arc, RwLock},
};

use crate::{
	dsnp::dsnp_configs::DsnpVersionConfig,
	graph::{
		key_manager::UserKeyManager, shared_state_manager::SharedStateManager, updates::UpdateEvent,
	},
	util::{
		time::time_in_ksecs,
		transactional_hashmap::{Transactional, TransactionalHashMap},
	},
};

use super::graph::Graph;
use log::Level;
use log_result_proc_macro::log_result_err;

/// Map of Graphs, keyed by SchemaId
pub type GraphMap = TransactionalHashMap<SchemaId, Graph>;

/// Structure to hold all of a User's Graphs, mapped by ConnectionType
#[derive(Debug, Clone)]
pub struct UserGraph {
	environment: Environment,
	graphs: GraphMap,
	update_tracker: UpdateTracker,
	pub user_key_manager: Arc<RwLock<UserKeyManager>>,
}

impl Transactional for UserGraph {
	fn commit(&mut self) {
		let ids: Vec<_> = self.graphs.inner().keys().copied().collect();
		for gid in ids {
			if let Some(g) = self.graphs.get_mut(&gid) {
				g.commit();
			}
		}
		self.graphs.commit();
		self.update_tracker.commit();
		self.user_key_manager.write().unwrap().commit();
	}

	fn rollback(&mut self) {
		self.graphs.rollback();
		let ids: Vec<_> = self.graphs.inner().keys().copied().collect();
		for gid in ids {
			if let Some(g) = self.graphs.get_mut(&gid) {
				g.rollback();
			}
		}
		self.update_tracker.rollback();
		self.user_key_manager.write().unwrap().rollback();
	}
}

impl UserGraph {
	/// Create a new, empty UserGraph
	pub fn new(
		user_id: &DsnpUserId,
		environment: &Environment,
		shared_state_manager: Arc<RwLock<SharedStateManager>>,
	) -> Self {
		let user_key_manager =
			Arc::new(RwLock::new(UserKeyManager::new(*user_id, shared_state_manager)));
		let graphs: GraphMap = environment
			.get_config()
			.schema_map
			.keys()
			.map(|schema_id| {
				(
					*schema_id,
					Graph::new(environment.clone(), *user_id, *schema_id, user_key_manager.clone()),
				)
			})
			.collect();

		Self {
			graphs,
			user_key_manager,
			update_tracker: UpdateTracker::new(),
			environment: environment.clone(),
		}
	}

	/// Getter for map of graphs
	pub fn graphs(&self) -> &GraphMap {
		&self.graphs
	}

	/// Getter for UpdateTracker
	pub fn update_tracker(&self) -> &UpdateTracker {
		&self.update_tracker
	}

	/// Getter for UpdateTracker
	pub fn update_tracker_mut(&mut self) -> &mut UpdateTracker {
		&mut self.update_tracker
	}

	/// Getter for UpdateTracker
	pub fn sync_updates(&mut self, schema_id: SchemaId) {
		let non_pending_connections: HashSet<DsnpUserId> = self
			.get_all_connections_of(schema_id, false)
			.iter()
			.map(|c| c.user_id)
			.collect();
		self.update_tracker.sync_updates(schema_id, &non_pending_connections);
	}

	/// Getter for the user's graph for the specified ConnectionType
	pub fn graph(&self, schema_id: &SchemaId) -> Option<&Graph> {
		self.graphs.get(schema_id)
	}

	/// Mutable getter for the user's graph for the specified ConnectionType
	pub fn graph_mut(&mut self, schema_id: &SchemaId) -> Option<&mut Graph> {
		self.graphs.get_mut(schema_id)
	}

	/// Setter for the specified graph connection type
	#[cfg(test)]
	pub fn set_graph(&mut self, schema_id: &SchemaId, graph: Graph) {
		self.graphs.insert(*schema_id, graph);
	}

	/// Clear the specified graph type for this user
	pub fn clear_graph(&mut self, schema_id: &SchemaId) {
		if let Some(g) = self.graphs.get_mut(schema_id) {
			g.clear();
		}
	}

	/// Calculate pending updates for all graphs for this user
	#[log_result_err(Level::Info)]
	pub fn calculate_updates(&self) -> DsnpGraphResult<Vec<Update>> {
		let mut result: Vec<Update> = Vec::new();
		for (schema_id, graph) in self.graphs.inner().iter() {
			if let Some(updates) = self.update_tracker.get_updates_for_schema_id(*schema_id) {
				let dsnp_version_config = self
					.get_dsnp_config(*schema_id)
					.ok_or(DsnpGraphError::UnsupportedSchema(*schema_id))?;

				let graph_data = graph.calculate_updates(&dsnp_version_config, &updates)?;
				result.extend(graph_data.into_iter());
			};
		}

		Ok(result)
	}

	// force calculates all imported graphs which will use the latest encryption key
	#[log_result_err(Level::Info)]
	pub fn force_calculate_graphs(&self) -> DsnpGraphResult<Vec<Update>> {
		let mut result = vec![];
		for (schema_id, graph) in self.graphs.inner().iter() {
			let dsnp_version_config = self
				.get_dsnp_config(*schema_id)
				.ok_or(DsnpGraphError::UnsupportedSchema(*schema_id))?;

			let updates = graph.force_recalculate(&dsnp_version_config)?;
			result.extend(updates);
		}

		Ok(result)
	}

	/// Check if graph with specified schema_id has a connection with the specified dsnp_user_id
	pub fn graph_has_connection(
		&self,
		schema_id: SchemaId,
		dsnp_user_id: DsnpUserId,
		include_pending: bool,
	) -> bool {
		let add_event = &UpdateEvent::Add { schema_id, dsnp_user_id };

		if let Some(graph) = self.graph(&schema_id) {
			let graph_connection_exists = graph.has_connection(&dsnp_user_id);
			let add_update_exists = include_pending && self.update_tracker.contains(add_event);
			let remove_update_exists =
				include_pending && self.update_tracker.contains_complement(add_event);

			return (graph_connection_exists && !remove_update_exists) ||
				(!graph_connection_exists && add_update_exists);
		}
		false
	}

	/// Get all the connections for the specified schema_id
	pub fn get_all_connections_of(
		&self,
		schema_id: SchemaId,
		apply_pending: bool,
	) -> Vec<DsnpGraphEdge> {
		let mut connections: HashSet<DsnpGraphEdge> = self
			.graphs
			.inner()
			.values()
			.filter(|graph| graph.get_schema_id() == schema_id)
			.flat_map(|graph| graph.pages().inner().values().map(|p| p.connections()))
			.flatten()
			.copied()
			.collect();

		if apply_pending {
			self.update_tracker
				.get_updates_for_schema_id(schema_id)
				.unwrap_or(&Vec::<UpdateEvent>::new())
				.iter()
				.cloned()
				.for_each(|event| match event {
					UpdateEvent::Add { dsnp_user_id, .. } => {
						connections.insert(DsnpGraphEdge {
							user_id: dsnp_user_id,
							since: time_in_ksecs(),
						});
					},
					UpdateEvent::Remove { dsnp_user_id, .. } => {
						connections.remove(&DsnpGraphEdge {
							user_id: dsnp_user_id,
							since: time_in_ksecs(),
						});
					},
				});
		}

		connections.into_iter().collect()
	}

	/// get dsnp config for a schema id
	pub fn get_dsnp_config(&self, schema_id: SchemaId) -> Option<DsnpVersionConfig> {
		let config = self.environment.get_config();
		if let Some(dsnp_version) = config.get_dsnp_version_from_schema_id(schema_id) {
			return Some(DsnpVersionConfig::new(dsnp_version));
		}
		None
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::{graph::key_manager::UserKeyProvider, iter_graph_connections, tests::helpers::*};
	use dryoc::keypair::StackKeyPair;
	use dsnp_graph_config::{GraphKeyType, ALL_CONNECTION_TYPES};

	#[test]
	fn new_creates_empty_graphs_for_all_connection_types() {
		let user_id = 1;
		let env = Environment::Mainnet;
		let user_graph =
			UserGraph::new(&user_id, &env, Arc::new(RwLock::new(SharedStateManager::new())));
		for c in ALL_CONNECTION_TYPES {
			let schema_id = env
				.get_config()
				.get_schema_id_from_connection_type(c)
				.expect("schema for connection type should exist");
			assert_eq!(user_graph.graphs.inner().contains_key(&schema_id), true);
		}

		for graph in user_graph.graphs.inner().values() {
			let graph_len = iter_graph_connections!(graph).len();
			assert_eq!(graph_len, 0);
		}
	}

	#[test]
	fn graph_getter_gets_correct_graph_for_connection_type() {
		let env = Environment::Mainnet;
		let user_graph = UserGraph::new(&1, &env, Arc::new(RwLock::new(SharedStateManager::new())));
		for c in ALL_CONNECTION_TYPES {
			let schema_id =
				env.get_config().get_schema_id_from_connection_type(c).expect("should exist");
			assert_eq!(user_graph.graph(&schema_id).unwrap().get_connection_type(), c);
		}
	}

	#[test]
	fn graph_mut_getter_gets_correct_graph_for_connection_type() {
		let env = Environment::Mainnet;
		let mut user_graph =
			UserGraph::new(&1, &env, Arc::new(RwLock::new(SharedStateManager::new())));
		for c in ALL_CONNECTION_TYPES {
			let schema_id =
				env.get_config().get_schema_id_from_connection_type(c).expect("should exist");
			assert_eq!(user_graph.graph_mut(&schema_id).unwrap().get_connection_type(), c);
		}
	}

	#[test]
	fn graph_setter_overwrites_existing_graph() {
		let env = Environment::Mainnet;
		let user_id = 1;
		let mut user_graph =
			UserGraph::new(&user_id, &env, Arc::new(RwLock::new(SharedStateManager::new())));
		let connection_type = ConnectionType::Follow(PrivacyType::Public);
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(connection_type)
			.expect("should exist");
		let mut new_graph = Graph::new(
			env,
			user_id,
			schema_id,
			Arc::new(RwLock::new(UserKeyManager::new(
				user_id,
				Arc::new(RwLock::new(SharedStateManager::new())),
			))),
		);
		assert_eq!(new_graph.add_connection_to_page(&0, &2).is_ok(), true);

		assert_ne!(*user_graph.graph(&schema_id).unwrap(), new_graph);
		user_graph.set_graph(&schema_id, new_graph.clone());
		assert_eq!(*user_graph.graph(&schema_id).unwrap(), new_graph);
	}

	#[test]
	fn clear_graph_clears_specific_graph_and_no_others() {
		let env = Environment::Mainnet;
		let graph = create_test_graph(None);
		let mut user_graph =
			UserGraph::new(&1, &env, Arc::new(RwLock::new(SharedStateManager::new())));
		for c in ALL_CONNECTION_TYPES {
			let schema_id =
				env.get_config().get_schema_id_from_connection_type(c).expect("should exist");
			user_graph.set_graph(&schema_id, graph.clone());
		}

		for (_, g) in user_graph.graphs.inner().iter() {
			assert_eq!(g.len(), 25);
		}

		let connection_type_to_clear = ConnectionType::Follow(PrivacyType::Public);
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(connection_type_to_clear)
			.expect("should exist");
		user_graph.clear_graph(&schema_id);

		for (schema_id_to_clear, g) in user_graph.graphs.inner() {
			if schema_id_to_clear == &schema_id {
				assert_eq!(g.len(), 0);
			} else {
				assert_eq!(g.len(), 25);
			}
		}
	}

	#[test]
	fn user_graph_rollback_should_revert_changes_on_user_and_underlying_graphs() {
		// arrange
		let env = Environment::Mainnet;
		let graph = create_test_graph(None);
		let mut user_graph =
			UserGraph::new(&1, &env, Arc::new(RwLock::new(SharedStateManager::new())));
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(ConnectionType::Follow(PrivacyType::Private))
			.expect("should exist");
		user_graph.set_graph(&schema_id, graph.clone());
		user_graph.commit();
		let connection_dsnp = 1000000;
		user_graph
			.update_tracker
			.register_update(UpdateEvent::Add { dsnp_user_id: connection_dsnp, schema_id }, false)
			.unwrap();
		let key = StackKeyPair::gen();
		user_graph
			.user_key_manager
			.write()
			.unwrap()
			.import_key_pairs(vec![GraphKeyPair {
				secret_key: key.secret_key.to_vec(),
				public_key: key.public_key.to_vec(),
				key_type: GraphKeyType::X25519,
			}])
			.unwrap();
		let graph = user_graph.graph_mut(&schema_id).unwrap();
		graph.add_connection_to_page(&1000, &connection_dsnp).unwrap();

		// act
		user_graph.rollback();

		// assert
		let graph = user_graph.graph(&schema_id).unwrap();
		assert!(graph.find_connection(&connection_dsnp).is_none());
		assert!(!user_graph.update_tracker.has_updates());
		assert_eq!(user_graph.user_key_manager.read().unwrap().get_imported_keys().len(), 0);
	}
}
