use crate::{constants, types::*};
use dryoc::keypair::StackKeyPair;
use dsnp_graph_config::{DsnpUserId, Environment, GraphKeyType, SchemaId};
use dsnp_graph_core::api::{
	api::{GraphAPI, GraphState},
	api_types::{
		Action, Connection, DsnpKeys, GraphKeyPair, ImportBundle, KeyData, PageData, Update,
	},
};
use rand::{prelude::SliceRandom, thread_rng, Rng};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use std::{
	collections::{HashMap, HashSet},
	fs::{File, OpenOptions},
	io::{Read, Write},
	vec,
};

pub fn choose_users(n: usize) -> HashSet<DsnpUserId> {
	let mut rng = rand::thread_rng();
	let mut users = HashSet::new();
	while users.len() < n {
		let user_id: DsnpUserId = rng.gen_range(1u32..constants::MAX_DSNP_USER_ID).into();
		users.insert(user_id);
	}
	users
}

pub fn choose_n_users_from(n: usize, users: &Vec<DsnpUserId>) -> HashSet<DsnpUserId> {
	let mut rng = rand::thread_rng();
	users.choose_multiple(&mut rng, n).cloned().collect()
}

pub fn choose_up_to_n_users_from(n: usize, users: &Vec<DsnpUserId>) -> HashSet<DsnpUserId> {
	let mut rng = rand::thread_rng();
	let exact_n: usize = rng.gen_range(0..=n).into();
	users.choose_multiple(&mut rng, exact_n).cloned().collect()
}

pub fn setup_initial_key(env: Environment, state: &mut GlobalState) {
	let public_key_schema_id = env.get_config().graph_public_key_schema_id;
	let keys: Vec<_> = state
		.users
		.par_iter()
		.map(|user_id| {
			let key_pair_raw = StackKeyPair::gen();
			let graph_key_pair = GraphKeyPair {
				secret_key: key_pair_raw.secret_key.to_vec(),
				public_key: key_pair_raw.public_key.to_vec(),
				key_type: GraphKeyType::X25519,
			};

			let mut graph = GraphState::new(env.clone());
			graph
				.apply_actions(
					&vec![Action::AddGraphKey {
						owner_dsnp_user_id: *user_id,
						new_public_key: graph_key_pair.public_key.clone(),
					}],
					&None,
				)
				.expect("error adding graph key");

			let mut dsnp_key = None;
			for a in graph.export_updates().expect("error exporting updates") {
				if let Update::AddKey { owner_dsnp_user_id, prev_hash, payload } = a {
					assert_eq!(dsnp_key, None);
					dsnp_key = Some(DsnpKeys {
						dsnp_user_id: owner_dsnp_user_id,
						keys_hash: prev_hash + 1,
						keys: vec![KeyData { index: 0, content: payload }],
					});
				}
			}

			println!("importing keys for user {}", user_id);
			(dsnp_key.unwrap(), graph_key_pair.clone())
		})
		.collect();

	for (dsnp, key_pair) in keys {
		let user_id = dsnp.dsnp_user_id;
		state.on_chain_keys.insert((user_id, public_key_schema_id), dsnp);
		state.wallet_keys.entry(user_id).or_default().push(key_pair);
	}
}

pub fn setup_initial_private_follows(
	env: Environment,
	max_connections: usize,
	private_follows_schema_id: SchemaId,
	state: &mut GlobalState,
) {
	let public_key_schema_id = env.get_config().graph_public_key_schema_id;

	let all_temp_data: Vec<TempData> = state
		.users
		.par_iter()
		.map(|user_id| {
			let mut rng = thread_rng();
			// random connection size
			let connection_size: usize = rng.gen_range(0..=max_connections).into();

			// choose connections
			let mut vec_connections: Vec<_> =
				state.users.choose_multiple(&mut rng, connection_size).cloned().collect();

			vec_connections.retain(|item| item != user_id);

			// get uploaded keys from chain
			let user_dsnp_keys = state
				.on_chain_keys
				.get(&(*user_id, public_key_schema_id))
				.expect("Should exist");
			// get key pairs from wallet
			let user_key_pairs = state.wallet_keys.get(user_id).expect("Should exist");
			// import user data
			let mut graph = GraphState::new(env.clone());
			graph
				.import_users_data(&vec![ImportBundle {
					schema_id: private_follows_schema_id,
					dsnp_user_id: *user_id,
					pages: vec![],
					dsnp_keys: Some(user_dsnp_keys.clone()),
					key_pairs: user_key_pairs.clone(),
				}])
				.expect("Should import");

			// create actions from connections
			let actions: Vec<_> = vec_connections
				.iter()
				.map(|c| Action::Connect {
					owner_dsnp_user_id: *user_id,
					connection: Connection {
						schema_id: private_follows_schema_id,
						dsnp_user_id: *c,
					},
					dsnp_keys: None,
				})
				.collect();
			// apply actions to state
			graph.apply_actions(&actions, &None).expect("Should import connections");

			// export state and apply to on chain graph
			let updates: Vec<_> = graph
				.export_updates()
				.expect(&format!("error exporting updates with {} connections", connection_size))
				.into_iter()
				.filter_map(|update| {
					if let Update::PersistPage { page_id, prev_hash, payload, .. } = update {
						return Some(PageData {
							page_id,
							content_hash: prev_hash + 1,
							content: payload,
						})
					}
					None
				})
				.collect();

			println!("{} pages created!", &updates.len());

			TempData {
				connections: vec_connections,
				pages: updates,
				user_id: *user_id,
				schema_id: private_follows_schema_id,
			}
		})
		.collect();

	for temp_data in all_temp_data {
		// insert into social graph
		state
			.social_graph
			.insert((temp_data.user_id, temp_data.schema_id), temp_data.connections);

		// insert into on chain graph
		state
			.on_chain_graph
			.insert((temp_data.user_id, temp_data.schema_id), temp_data.pages);
	}
}

pub fn setup_initial_private_friendships(
	env: Environment,
	max_connections: usize,
	private_friendship_schema_id: SchemaId,
	state: &mut GlobalState,
) {
	// select friendship users
	let mut temp_graph: HashMap<DsnpUserId, Vec<DsnpUserId>> = HashMap::new();
	let selected_users: Vec<_> =
		choose_n_users_from(state.users.len() / 2, &state.users).into_iter().collect();
	for u in selected_users.iter() {
		temp_graph.insert(*u, vec![]);
	}

	println!("{} users are selected for private friendship", selected_users.len());

	for i in 0..=selected_users.len() / 2 {
		let my_user = *selected_users.get(i).unwrap();
		let current_friends = temp_graph.get(&my_user).unwrap();
		let mut new_friends: Vec<_> = choose_up_to_n_users_from(
			max_connections - current_friends.len(),
			&selected_users[(i + 1)..].to_vec(),
		)
		.into_iter()
		.collect();
		new_friends.retain(|item| *item != my_user && !current_friends.contains(item));

		temp_graph.entry(my_user).or_insert(vec![]).extend(new_friends.iter());
		for f in new_friends {
			temp_graph.entry(f).or_insert(vec![]).push(my_user);
		}
	}

	let all_temp_data: Vec<TempData> = selected_users
		.par_iter()
		.map(|user_id| {
			// get uploaded keys from chain
			let (user_dsnp_keys, user_key_pairs, pages, _) =
				state.get_all_data_for_user(env.clone(), *user_id, private_friendship_schema_id);
			// prepare all bundles
			let mut all_bundles: Vec<_> = temp_graph
				.get(user_id)
				.unwrap()
				.iter()
				.map(|c| {
					let (friend_dsnp_keys, friend_key_pairs, _, _) =
						state.get_all_data_for_user(env.clone(), *c, private_friendship_schema_id);
					ImportBundle {
						schema_id: private_friendship_schema_id,
						dsnp_user_id: *c,
						pages: vec![],
						dsnp_keys: Some(friend_dsnp_keys.clone()),
						key_pairs: friend_key_pairs.clone(),
					}
				})
				.collect();
			all_bundles.push(ImportBundle {
				schema_id: private_friendship_schema_id,
				dsnp_user_id: *user_id,
				pages,
				dsnp_keys: Some(user_dsnp_keys.clone()),
				key_pairs: user_key_pairs.clone(),
			});
			// import user data
			let mut graph = GraphState::new(env.clone());
			graph.import_users_data(&all_bundles).expect("Should import");
			// create actions from connections
			let actions: Vec<_> = temp_graph
				.get(user_id)
				.unwrap()
				.iter()
				.map(|c| Action::Connect {
					owner_dsnp_user_id: *user_id,
					connection: Connection {
						schema_id: private_friendship_schema_id,
						dsnp_user_id: *c,
					},
					dsnp_keys: None,
				})
				.collect();
			// apply actions to state
			graph.apply_actions(&actions, &None).expect("Should import connections");

			// export state and apply to on chain graph
			let updates: Vec<_> = graph
				.export_updates()
				.expect(&format!("error exporting updates for user {}", user_id))
				.into_iter()
				.filter_map(|update| {
					if let Update::PersistPage { page_id, prev_hash, payload, .. } = update {
						return Some(PageData {
							page_id,
							content_hash: prev_hash + 1,
							content: payload,
						})
					}
					None
				})
				.collect();

			println!(
				"{} pages created {} users!",
				&updates.len(),
				temp_graph.get(user_id).unwrap().len()
			);

			TempData {
				connections: temp_graph.get(user_id).unwrap().clone(),
				pages: updates,
				user_id: *user_id,
				schema_id: private_friendship_schema_id,
			}
		})
		.collect();

	for temp_data in all_temp_data {
		// insert into social graph
		state
			.social_graph
			.insert((temp_data.user_id, temp_data.schema_id), temp_data.connections);

		// insert into on chain graph
		state
			.on_chain_graph
			.insert((temp_data.user_id, temp_data.schema_id), temp_data.pages);
	}
}

#[warn(dead_code)]
fn _append_to_temp(data: Vec<TempData>) {
	let mut data_file = match OpenOptions::new().append(true).open("temp.bin") {
		Ok(f) => f,
		Err(_) => File::create_new("temp.bin").expect("Should be able to create new file"),
	};

	for d in data {
		let encoded: Vec<u8> = bincode::serialize(&d).expect("TempData encoding should work!");
		data_file.write(&encoded).expect("Writing to temp should work!");
	}

	data_file.flush().expect("Flush should work!");
}

#[warn(dead_code)]
fn _read_temp() -> Vec<TempData> {
	let data_file = OpenOptions::new().read(true).open("temp.bin");
	match data_file {
		Err(_) => {
			println!("No file to open for temp!");
			return vec![]
		},
		Ok(mut file) => {
			let mut result = vec![];

			let mut buffer = vec![];
			file.read_to_end(&mut buffer).expect("Read to end should work");

			let mut slice = &buffer[..];
			while !slice.is_empty() {
				let temp: TempData = bincode::deserialize(&slice).expect("should deserialize");
				let my_serialized = bincode::serialize(&temp).expect("should serialize");
				slice = &slice[my_serialized.len()..];

				result.push(temp);
			}

			return result
		},
	}
}
