use crate::{
	constants::{self, PRIVATE_FRIENDSHIP_PAGE_MODIFICATIONS},
	scenarions::common::{add_keys_for_users, compare_on_chain_with_expected, modify_random_pages},
	GlobalState,
};
use dsnp_graph_config::Environment;
use dsnp_graph_core::api::api_types::*;
use rand::{prelude::SliceRandom, thread_rng};

pub fn execute_key_rotation_private_follow(state: &mut GlobalState, env: Environment) {
	let mut rng = thread_rng();
	let private_follow_schema_id = env
		.get_config()
		.get_schema_id_from_connection_type(ConnectionType::Follow(PrivacyType::Private))
		.unwrap();
	let selected_users: Vec<_> = state
		.users
		.choose_multiple(&mut rng, constants::KEY_ROTATIONS)
		.cloned()
		.collect();

	add_keys_for_users(env.clone(), state, &selected_users, private_follow_schema_id);

	modify_random_pages(env.clone(), state, &selected_users, private_follow_schema_id, false);

	compare_on_chain_with_expected(env.clone(), state, None, private_follow_schema_id);

	println!("Success: All graphs matched after execute_key_rotation_private_follow!");
}

pub fn execute_key_rotation_private_friendship(state: &mut GlobalState, env: Environment) {
	let mut rng = thread_rng();
	let private_friendship_schema_id = env
		.get_config()
		.get_schema_id_from_connection_type(ConnectionType::Friendship(PrivacyType::Private))
		.unwrap();
	let selected_users: Vec<_> = state
		.get_all_users_in_graph_for(private_friendship_schema_id)
		.choose_multiple(&mut rng, constants::KEY_ROTATIONS)
		.cloned()
		.collect();

	add_keys_for_users(env.clone(), state, &selected_users, private_friendship_schema_id);

	compare_on_chain_with_expected(
		env.clone(),
		state,
		Some(&selected_users),
		private_friendship_schema_id,
	);

	let modification_users: Vec<_> = selected_users
		.choose_multiple(&mut rng, PRIVATE_FRIENDSHIP_PAGE_MODIFICATIONS)
		.copied()
		.collect();
	modify_random_pages(
		env.clone(),
		state,
		&modification_users,
		private_friendship_schema_id,
		true,
	);

	compare_on_chain_with_expected(
		env.clone(),
		state,
		None,
		private_friendship_schema_id,
	);

	println!("Success: All graphs matched after execute_key_rotation_private_friendship!");
}
