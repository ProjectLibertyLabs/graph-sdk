use crate::{
	dsnp::{api_types::*, dsnp_types::*},
	graph::updates::UpdateTracker,
};
use anyhow::Result;
use std::collections::HashMap;

use crate::dsnp::encryption::EncryptionBehavior;

use super::graph::Graph;

pub type GraphMap = HashMap<ConnectionType, Graph>;

/// Structure to hold all of a User's Graphs, mapped by ConnectionType
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserGraph {
	user_id: DsnpUserId,
	graphs: GraphMap,
	pub update_tracker: UpdateTracker,
}

impl UserGraph {
	/// Create a new, empty UserGraph
	pub fn new(user_id: &DsnpUserId) -> Self {
		Self {
			user_id: *user_id,
			graphs: GraphMap::from([
				(
					ConnectionType::Follow(PrivacyType::Public),
					Graph::new(ConnectionType::Follow(PrivacyType::Public)),
				),
				(
					ConnectionType::Follow(PrivacyType::Private),
					Graph::new(ConnectionType::Follow(PrivacyType::Private)),
				),
				(
					ConnectionType::Friendship(PrivacyType::Public),
					Graph::new(ConnectionType::Friendship(PrivacyType::Public)),
				),
				(
					ConnectionType::Friendship(PrivacyType::Private),
					Graph::new(ConnectionType::Friendship(PrivacyType::Private)),
				),
			]),
			update_tracker: UpdateTracker::new(),
		}
	}

	/// Getter for map of graphs
	pub fn graphs(&self) -> &GraphMap {
		&self.graphs
	}

	/// Getter for UpdateTracker
	pub fn update_tracker(&mut self) -> &mut UpdateTracker {
		&mut self.update_tracker
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

	/// Cacluate pending updates
	pub fn calculate_updates<E: EncryptionBehavior>(
		&mut self,
		connection_keys: &Vec<DsnpKeys<E>>,
		encryption_key: (u64, &PublicKey<E>),
	) -> Result<Vec<ExportBundle>> {
		self.graphs
			.iter()
			.map(|(connection_type, graph)| {
				graph.calculate_updates(
					self.update_tracker.get_mut_updates_for_connection_type(*connection_type),
					&self.user_id,
					connection_keys,
					encryption_key,
				)
			})
			.collect()
	}
}

#[cfg(test)]
mod test {
	use crate::{graph::test_helpers::create_test_graph, iter_graph_connections};

	use super::*;

	#[test]
	fn new_creates_empty_graphs_for_all_connection_types() {
		let user_id = 1;
		let user_graph = UserGraph::new(&user_id);

		assert_eq!(user_graph.user_id, user_id);
		assert_eq!(
			user_graph.graphs.contains_key(&ConnectionType::Follow(PrivacyType::Public)),
			true
		);
		assert_eq!(
			user_graph.graphs.contains_key(&ConnectionType::Follow(PrivacyType::Private)),
			true
		);
		assert_eq!(
			user_graph.graphs.contains_key(&ConnectionType::Friendship(PrivacyType::Public)),
			true
		);
		assert_eq!(
			user_graph
				.graphs
				.contains_key(&ConnectionType::Friendship(PrivacyType::Private)),
			true
		);

		for graph in user_graph.graphs.values() {
			let graph_len = iter_graph_connections!(graph).len();
			assert_eq!(graph_len, 0);
		}
	}

	#[test]
	fn graph_getter_gets_correct_graph_for_connection_type() {
		let user_graph = UserGraph::new(&1);
		for p in [PrivacyType::Public, PrivacyType::Private] {
			for c in [ConnectionType::Follow(p), ConnectionType::Friendship(p)] {
				assert_eq!(user_graph.graph(&c).connection_type, c);
			}
		}
	}

	#[test]
	fn graph_mut_getter_gets_correct_graph_for_connection_type() {
		let mut user_graph = UserGraph::new(&1);
		for p in [PrivacyType::Public, PrivacyType::Private] {
			for c in [ConnectionType::Follow(p), ConnectionType::Friendship(p)] {
				assert_eq!(user_graph.graph_mut(&c).connection_type, c);
			}
		}
	}

	#[test]
	fn graph_setter_overwrites_existing_graph() {
		let mut user_graph = UserGraph::new(&1);
		let connection_type = ConnectionType::Follow(PrivacyType::Public);
		let mut new_graph = Graph::new(connection_type);
		assert_eq!(new_graph.add_connection_to_page(&0, &2).is_ok(), true);

		assert_ne!(*user_graph.graph(&connection_type), new_graph);
		user_graph.set_graph(&connection_type, new_graph.clone());
		assert_eq!(*user_graph.graph(&connection_type), new_graph);
	}

	#[test]
	fn clear_graph_clears_specific_graph_and_no_others() {
		let graph = create_test_graph();
		let mut user_graph = UserGraph::new(&1);
		for p in [PrivacyType::Public, PrivacyType::Private] {
			for c in [ConnectionType::Follow(p), ConnectionType::Friendship(p)] {
				user_graph.set_graph(&c, graph.clone());
			}
		}

		for (_, g) in user_graph.graphs.iter() {
			assert_eq!(g.len(), 25);
		}

		let connection_type_to_clear = ConnectionType::Follow(PrivacyType::Public);
		user_graph.clear_graph(&connection_type_to_clear);

		for (conn_type, g) in user_graph.graphs {
			if conn_type == connection_type_to_clear {
				assert_eq!(g.len(), 0);
			} else {
				assert_eq!(g.len(), 25);
			}
		}
	}

	#[test]
	fn clear_all_clears_all_graphs() {
		let graph = create_test_graph();
		let mut user_graph = UserGraph::new(&1);
		for p in [PrivacyType::Public, PrivacyType::Private] {
			for c in [ConnectionType::Follow(p), ConnectionType::Friendship(p)] {
				user_graph.set_graph(&c, graph.clone());
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
