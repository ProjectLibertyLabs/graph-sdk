use std::collections::BTreeSet;

use dryoc::keypair::StackKeyPair;
use dsnp_graph_config::{DsnpUserId, Environment, GraphKeyType, SchemaId};
use dsnp_graph_core::api::{
	api::{GraphAPI, GraphState},
	api_types::{Action, Connection, GraphKeyPair, ImportBundle},
};
use rand::{prelude::SliceRandom, thread_rng, Rng};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{constants, GlobalState};

pub fn add_keys_for_users(
	env: Environment,
	state: &mut GlobalState,
	selected_users: &Vec<DsnpUserId>,
	schema_id: SchemaId,
) {
	let keys_updates: Vec<_> = selected_users
		.par_iter()
		.map(|user_id| {
			let key_pair_raw = StackKeyPair::gen();
			let graph_key_pair = GraphKeyPair {
				secret_key: key_pair_raw.secret_key.to_vec(),
				public_key: key_pair_raw.public_key.to_vec(),
				key_type: GraphKeyType::X25519,
			};

			let (existing_keys, key_pairs, pages, _) =
				state.get_all_data_for_user(env.clone(), *user_id, schema_id);
			let mut graph = GraphState::new(env.clone());
			graph
				.import_users_data(&vec![ImportBundle {
					dsnp_keys: Some(existing_keys.clone()),
					dsnp_user_id: *user_id,
					schema_id,
					key_pairs,
					pages,
				}])
				.expect("Should import data");
			graph
				.apply_actions(
					&vec![Action::AddGraphKey {
						owner_dsnp_user_id: *user_id,
						new_public_key: graph_key_pair.public_key.clone(),
					}],
					&None,
				)
				.expect("error adding graph key");

			let updates = graph.export_updates().expect("error exporting updates");

			println!("importing keys for user {}", user_id);
			(*user_id, updates, graph_key_pair)
		})
		.collect();

	for (user_id, updates, graph_key_pair) in keys_updates {
		state.apply_updates_for_user(
			env.clone(),
			user_id,
			schema_id,
			&updates,
			&vec![],
			&vec![],
			Some(&graph_key_pair),
		);
	}
}

pub fn modify_random_pages(
	env: Environment,
	state: &mut GlobalState,
	selected_users: &[u64],
	schema_id: SchemaId,
	is_friendship: bool,
) {
	let public_key_schema_id = env.get_config().graph_public_key_schema_id;
	let changes: Vec<_> = selected_users
		.par_iter()
		.map(|user_id| {
			// get all user data
			let (imports, social_graph) =
				state.prepare_all_import_bundles(env.clone(), *user_id, schema_id, is_friendship);
			// import user data
			let mut graph = GraphState::new(env.clone());
			graph.import_users_data(&imports).expect("Should import");
			// assert graphs with expected
			let graph_edges = graph
				.get_connections_for_user_graph(user_id, &schema_id, false)
				.expect("Should get conections");
			let graph_users_set: BTreeSet<_> =
				graph_edges.clone().iter().map(|e| e.user_id).collect();
			let social_graph_set: BTreeSet<_> = social_graph.iter().map(|c| *c).collect();
			assert_eq!(graph_users_set, social_graph_set, "graphs should match for {}", user_id);
			// choose random number of operations for add and removed
			let mut rng = thread_rng();
			let graph_users: Vec<_> = graph_users_set.clone().into_iter().collect();
			let remove_size: usize = rng.gen_range(0..=(graph_users.len() / 2)).into();
			let add_size: usize =
				rng.gen_range(0..=((constants::CONNECTIONS - graph_users.len()) / 2)).into();
			// removal operation
			let connections_to_remove: Vec<_> =
				graph_users.choose_multiple(&mut rng, remove_size).copied().collect();
			let mut actions: Vec<_> = connections_to_remove
				.iter()
				.map(|c| Action::Disconnect {
					owner_dsnp_user_id: *user_id,
					connection: Connection { dsnp_user_id: *c, schema_id },
				})
				.collect();

			// add operation
			let mut connections_to_add: Vec<_> =
				state.users.choose_multiple(&mut rng, add_size).cloned().collect();
			connections_to_add.retain(|item| item != user_id && !social_graph.contains(item));
			let add_actions: Vec<_> = connections_to_add
				.clone()
				.iter()
				.map(|c| {
					let dsnp_keys = if is_friendship {
						Some(
							state
								.on_chain_keys
								.get(&(*c, public_key_schema_id))
								.expect("Should key exist")
								.clone(),
						)
					} else {
						None
					};

					Action::Connect {
						owner_dsnp_user_id: *user_id,
						connection: Connection { dsnp_user_id: *c, schema_id },
						dsnp_keys,
					}
				})
				.collect();

			actions.extend(add_actions);
			actions.shuffle(&mut rng);
			graph.apply_actions(&actions, &None).expect("Should apply removals");

			// get result
			println!("modifying user {} graph", user_id);
			let updates = graph.export_updates().expect("Should work without issues");
			(*user_id, connections_to_add, connections_to_remove, updates)
		})
		.collect();

	// apply all changes
	for (user_id, adds, removes, updates) in changes {
		state.apply_updates_for_user(
			env.clone(),
			user_id,
			schema_id,
			&updates,
			&adds,
			&removes,
			None,
		);
	}
}

pub fn compare_on_chain_with_expected(
	env: Environment,
	state: &GlobalState,
	selected_users: Option<&[u64]>,
	schema_id: SchemaId,
) {
	let users = selected_users.unwrap_or(&state.users);
	users.par_iter().for_each(|user_id| {
		// get all user data
		let (user_dsnp_keys, user_key_pairs, user_pages, social_graph) =
			state.get_all_data_for_user(env.clone(), *user_id, schema_id);
		// create graph state
		let mut graph = GraphState::new(env.clone());
		graph
			.import_users_data(&vec![ImportBundle {
				schema_id,
				dsnp_user_id: *user_id,
				pages: user_pages.clone(),
				dsnp_keys: Some(user_dsnp_keys.clone()),
				key_pairs: user_key_pairs.clone(),
			}])
			.expect("Should import");
		// assert graphs with expected
		let graph_edges = graph
			.get_connections_for_user_graph(user_id, &schema_id, false)
			.expect("Should get conections");
		let graph_users_set: BTreeSet<_> = graph_edges.clone().iter().map(|e| e.user_id).collect();
		let social_graph_set: BTreeSet<_> = social_graph.iter().map(|c| *c).collect();
		assert_eq!(
			graph_edges.len(),
			social_graph.len(),
			"graph sizes should match for {}",
			user_id
		);
		assert_eq!(graph_users_set, social_graph_set, "graphs should match for {}", user_id);
	});
}
