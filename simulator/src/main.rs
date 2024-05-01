use dsnp_graph_config::{ConnectionType, Environment, PrivacyType};
use scenarions::key_rotation::execute_key_rotation_private_friendship;
use std::{
	fs::File,
	io::{Read, Write},
};
mod constants;
mod init;
mod scenarions;
mod types;

use crate::{scenarions::key_rotation::execute_key_rotation_private_follow, types::*};

fn main() {
	let env = Environment::Mainnet;
	let mut state: GlobalState = match File::open(constants::STATE_FILE) {
		Ok(mut file) => {
			let mut buffer = Vec::<u8>::new();
			file.read_to_end(&mut buffer).expect("Read to end should work");
			bincode::deserialize(&buffer).unwrap_or_default()
		},
		Err(_) => GlobalState::default(),
	};

	init_state_machine(&mut state, env.clone());

	execute_key_rotation_private_follow(&mut state, env.clone());

	execute_key_rotation_private_friendship(&mut state, env);
}

fn init_state_machine(state: &mut GlobalState, env: Environment) {
	while state.current_status.clone() != Status::End {
		let current_status = state.current_status.clone();
		println!("current status {:?}", &current_status);
		match current_status {
			Status::Init => {
				// 1. choose all users
				state.users = init::choose_users(constants::USERS).into_iter().collect();
				state.current_status = Status::InitialUsersCreated;
				persist_state(state);
			},
			Status::InitialUsersCreated => {
				// 2. setup initial key for all users
				init::setup_initial_key(env.clone(), state);
				state.current_status = Status::InitialKeysCreated;
				persist_state(state);
			},
			Status::InitialKeysCreated => {
				// 3. setup initial private follows
				let private_follow_schema_id = env
					.get_config()
					.get_schema_id_from_connection_type(ConnectionType::Follow(
						PrivacyType::Private,
					))
					.unwrap();

				init::setup_initial_private_follows(
					env.clone(),
					constants::CONNECTIONS,
					private_follow_schema_id,
					state,
				);

				state.current_status = Status::PrivateFollowsCreated;
				persist_state(state);
				println!("All private follows are created!");
			},
			Status::PrivateFollowsCreated => {
				// 4. setup initial private friendships
				let private_friendship_schema_id = env
					.get_config()
					.get_schema_id_from_connection_type(ConnectionType::Friendship(
						PrivacyType::Private,
					))
					.unwrap();

				init::setup_initial_private_friendships(
					env.clone(),
					constants::CONNECTIONS,
					private_friendship_schema_id,
					state,
				);

				state.current_status = Status::PrivateFriendshipsCreated;
				persist_state(state);
				println!("All private friendships are created!");
			},
			Status::PrivateFriendshipsCreated => {
				state.current_status = Status::End;
			},
			Status::End => {
				println!("All connections are created!");
			},
		}
	}

	println!("Initializing is done!");
}

fn persist_state(state: &GlobalState) {
	let encoded: Vec<u8> = bincode::serialize(&state).unwrap();
	let mut file = File::create(constants::STATE_FILE).expect("Should open file");
	file.write_all(&encoded[..]).expect("Should write data");
}
