use crate::{
	dsnp::{api_types::*, dsnp_types::*},
	graph::updates::UpdateTracker,
};
use anyhow::{Error, Result};
use dsnp_graph_config::{Environment, SchemaId};
use std::{
	cell::RefCell,
	collections::{HashMap, HashSet},
	rc::Rc,
};

use crate::{
	dsnp::dsnp_configs::DsnpVersionConfig,
	graph::key_manager::{PublicKeyManager, UserKeyManager, UserKeyProvider},
};

use super::graph::Graph;

pub type GraphMap = HashMap<SchemaId, Graph>;

/// Structure to hold all of a User's Graphs, mapped by ConnectionType
#[derive(Debug)]
pub struct UserGraph {
	user_id: DsnpUserId,
	graphs: GraphMap,
	user_key_manager: UserKeyManager,
	update_tracker: UpdateTracker,
}

impl UserGraph {
	/// Create a new, empty UserGraph
	pub fn new(
		user_id: &DsnpUserId,
		environment: &Environment,
		public_key_manager: Rc<RefCell<PublicKeyManager>>,
	) -> Self {
		let graphs: GraphMap = environment
			.get_config()
			.schema_map
			.keys()
			.map(|schema_id| (*schema_id, Graph::new(environment.clone(), *schema_id)))
			.collect();

		Self {
			user_id: *user_id,
			graphs,
			user_key_manager: UserKeyManager::new(*user_id, public_key_manager),
			update_tracker: UpdateTracker::new(),
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

	/// Getter for UserKeyManager
	pub fn user_key_manager(&self) -> &UserKeyManager {
		&self.user_key_manager
	}

	/// Mutable Getter for UserKeyManager
	pub fn user_key_manager_mut(&mut self) -> &mut UserKeyManager {
		&mut self.user_key_manager
	}

	/// Getter for the user's graph for the specified ConnectionType
	pub fn graph(&self, schema_id: &SchemaId) -> &Graph {
		self.graphs.get(schema_id).expect("UserGraph local instance is corrupt")
	}

	/// Mutable getter for the user's graph for the specified ConnectionType
	pub fn graph_mut(&mut self, schema_id: &SchemaId) -> &mut Graph {
		self.graphs.get_mut(schema_id).expect("UserGraph local instance is corrupt")
	}

	/// Setter for the specified graph connection type
	pub fn set_graph(&mut self, schema_id: &SchemaId, graph: Graph) {
		self.graphs.insert(*schema_id, graph);
	}

	/// Clear the specified graph type for this user
	pub fn clear_graph(&mut self, schema_id: &SchemaId) {
		if let Some(g) = self.graphs.get_mut(schema_id) {
			g.clear();
		}
	}

	/// Clear all graphs associated with this user
	pub fn clear_all(&mut self) {
		self.graphs.iter_mut().for_each(|(_, g)| g.clear());
	}

	/// Calculate pending updates for all graphs for this user
	pub fn calculate_updates(
		&mut self,
		dsnp_version_config: &DsnpVersionConfig,
	) -> Result<Vec<Update>> {
		let mut result: Vec<Update> = Vec::new();
		let (public_key, keypair) = self
			.user_key_manager
			.get_resolved_active_key(self.user_id)
			.ok_or(Error::msg("No resolved active key found!"))?;
		// TODO: calculate PRIds
		let prids = vec![];
		for (schema_id, graph) in self.graphs.iter() {
			let graph_data = graph.calculate_updates(
				dsnp_version_config,
				self.update_tracker.get_mut_updates_for_schema_id(*schema_id),
				&self.user_id,
				&prids,
				&ResolvedKeyPair { key_id: public_key.key_id.unwrap(), key_pair: keypair.clone() },
			)?;
			result.extend(graph_data.into_iter());
		}

		Ok(result)
	}

	pub fn get_all_connections_of(&self, connection_type: ConnectionType) -> Vec<DsnpUserId> {
		let result: HashSet<DsnpUserId> = self
			.graphs
			.values()
			.filter(|graph| graph.get_connection_type() == connection_type)
			.flat_map(|graph| graph.pages().values().map(|p| p.connections()))
			.flatten()
			.map(|c| c.user_id)
			.collect();

		result.into_iter().collect()
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::{iter_graph_connections, tests::helpers::*};

	#[test]
	fn new_creates_empty_graphs_for_all_connection_types() {
		let user_id = 1;
		let env = Environment::Mainnet;
		let user_graph =
			UserGraph::new(&user_id, &env, Rc::new(RefCell::from(PublicKeyManager::new())));

		assert_eq!(user_graph.user_id, user_id);
		let follow_public_schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(ConnectionType::Follow(PrivacyType::Public))
			.expect("should exist");
		assert_eq!(user_graph.graphs.contains_key(&follow_public_schema_id), true);
		let follow_private_schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(ConnectionType::Follow(PrivacyType::Private))
			.expect("should exist");
		assert_eq!(user_graph.graphs.contains_key(&follow_private_schema_id), true);
		let friendship_public_schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(ConnectionType::Friendship(PrivacyType::Public))
			.expect("should exist");
		assert_eq!(user_graph.graphs.contains_key(&friendship_public_schema_id), true);
		let friendship_private_schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(ConnectionType::Friendship(PrivacyType::Private))
			.expect("should exist");
		assert_eq!(user_graph.graphs.contains_key(&friendship_private_schema_id), true);

		for graph in user_graph.graphs.values() {
			let graph_len = iter_graph_connections!(graph).len();
			assert_eq!(graph_len, 0);
		}
	}

	#[test]
	fn graph_getter_gets_correct_graph_for_connection_type() {
		let env = Environment::Mainnet;
		let user_graph = UserGraph::new(&1, &env, Rc::new(RefCell::from(PublicKeyManager::new())));
		for p in [PrivacyType::Public, PrivacyType::Private] {
			for c in [ConnectionType::Follow(p), ConnectionType::Friendship(p)] {
				let schema_id =
					env.get_config().get_schema_id_from_connection_type(c).expect("should exist");
				assert_eq!(user_graph.graph(&schema_id).get_connection_type(), c);
			}
		}
	}

	#[test]
	fn graph_mut_getter_gets_correct_graph_for_connection_type() {
		let env = Environment::Mainnet;
		let mut user_graph =
			UserGraph::new(&1, &env, Rc::new(RefCell::from(PublicKeyManager::new())));
		for p in [PrivacyType::Public, PrivacyType::Private] {
			for c in [ConnectionType::Follow(p), ConnectionType::Friendship(p)] {
				let schema_id =
					env.get_config().get_schema_id_from_connection_type(c).expect("should exist");
				assert_eq!(user_graph.graph_mut(&schema_id).get_connection_type(), c);
			}
		}
	}

	#[test]
	fn graph_setter_overwrites_existing_graph() {
		let env = Environment::Mainnet;
		let mut user_graph =
			UserGraph::new(&1, &env, Rc::new(RefCell::from(PublicKeyManager::new())));
		let connection_type = ConnectionType::Follow(PrivacyType::Public);
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(connection_type)
			.expect("should exist");
		let mut new_graph = Graph::new(env, schema_id);
		assert_eq!(new_graph.add_connection_to_page(&0, &2).is_ok(), true);

		assert_ne!(*user_graph.graph(&schema_id), new_graph);
		user_graph.set_graph(&schema_id, new_graph.clone());
		assert_eq!(*user_graph.graph(&schema_id), new_graph);
	}

	#[test]
	fn clear_graph_clears_specific_graph_and_no_others() {
		let env = Environment::Mainnet;
		let graph = create_test_graph();
		let mut user_graph =
			UserGraph::new(&1, &env, Rc::new(RefCell::from(PublicKeyManager::new())));
		for p in [PrivacyType::Public, PrivacyType::Private] {
			for c in [ConnectionType::Follow(p), ConnectionType::Friendship(p)] {
				let schema_id =
					env.get_config().get_schema_id_from_connection_type(c).expect("should exist");
				user_graph.set_graph(&schema_id, graph.clone());
			}
		}

		for (_, g) in user_graph.graphs.iter() {
			assert_eq!(g.len(), 25);
		}

		let connection_type_to_clear = ConnectionType::Follow(PrivacyType::Public);
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(connection_type_to_clear)
			.expect("should exist");
		user_graph.clear_graph(&schema_id);

		for (schema_id_to_clear, g) in user_graph.graphs {
			if schema_id_to_clear == schema_id {
				assert_eq!(g.len(), 0);
			} else {
				assert_eq!(g.len(), 25);
			}
		}
	}

	#[test]
	fn clear_all_clears_all_graphs() {
		let env = Environment::Mainnet;
		let graph = create_test_graph();
		let mut user_graph =
			UserGraph::new(&1, &env, Rc::new(RefCell::from(PublicKeyManager::new())));
		for p in [PrivacyType::Public, PrivacyType::Private] {
			for c in [ConnectionType::Follow(p), ConnectionType::Friendship(p)] {
				let schema_id =
					env.get_config().get_schema_id_from_connection_type(c).expect("should exist");
				user_graph.set_graph(&schema_id, graph.clone());
			}
		}

		for (_, g) in user_graph.graphs.iter() {
			assert_eq!(g.len(), 25);
		}

		user_graph.clear_all();

		for (_, g) in user_graph.graphs.iter() {
			assert_eq!(g.len(), 0);
		}
	}
}
