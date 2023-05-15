mod common;

use crate::common::get_schema_from;
use dsnp_graph_config::{builder::ConfigBuilder, ConnectionType, Environment, PrivacyType};
use dsnp_graph_core::{
	api::api::{GraphAPI, GraphState},
	dsnp::api_types::{PageData, PageId},
	util::builders::ImportBundleBuilder,
};

#[cfg(test)]
mod integration_tests {
	use super::*;
	use dryoc::keypair::StackKeyPair;
	use dsnp_graph_config::{DsnpVersion, GraphKeyType, SchemaConfig};
	use dsnp_graph_core::dsnp::{
		api_types::{Action, Connection, GraphKeyPair, ResolvedKeyPair},
		dsnp_configs::KeyPairType,
		dsnp_types::{DsnpGraphEdge, DsnpPrid, DsnpPublicKey, DsnpUserId},
		pseudo_relationship_identifier::PridProvider,
	};
	use std::{borrow::Borrow, collections::HashSet};

	#[test]
	fn state_capacity_should_return_correct_capacity() {
		// arrange
		let capacity: usize = 10;
		let env = Environment::Dev(
			ConfigBuilder::new().with_sdk_max_users_graph_size(capacity as u32).build(),
		);
		let state = GraphState::new(env);

		// act
		let state_capacity = state.capacity();

		// assert
		assert_eq!(state_capacity, capacity);
	}

	#[test]
	fn state_with_capacity_of_lower_than_env_should_return_smaller_capacity() {
		// arrange
		let capacity: usize = 10;

		// act
		let state = GraphState::with_capacity(Environment::Mainnet, capacity);

		// assert
		assert_eq!(state.capacity(), capacity);
	}

	#[test]
	fn state_with_capacity_of_higher_than_env_should_return_smaller_capacity() {
		// arrange
		let capacity: usize = 10000;
		let env = Environment::Mainnet;

		// act
		let state = GraphState::with_capacity(env.clone(), capacity);

		// assert
		assert_eq!(state.capacity(), env.get_config().sdk_max_users_graph_size as usize);
	}

	#[test]
	fn api_len_with_empty_state_should_be_zero() {
		// arrange
		let state = GraphState::new(Environment::Mainnet);

		// act
		let len = state.len();

		// assert
		assert_eq!(len, 0);
	}

	#[test]
	fn api_import_user_data_for_public_follow_should_import_graph_successfully() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id = get_schema_from(env.clone(), ConnectionType::Follow(PrivacyType::Public));
		let mut state = GraphState::new(env.clone());
		let dsnp_user_id_1 = 1;
		let dsnp_user_id_2 = 2;
		let connections_1 = vec![(2, 0), (3, 0), (4, 0), (5, 0)];
		let connections_2 = vec![(10, 0), (11, 0), (12, 0), (13, 0)];
		let input1 = ImportBundleBuilder::new(env.clone(), dsnp_user_id_1, schema_id)
			.with_page(1, &connections_1, &vec![], 0)
			.build();
		let input2 = ImportBundleBuilder::new(env, dsnp_user_id_2, schema_id)
			.with_page(1, &connections_2, &vec![], 0)
			.build();

		// act
		let res = state.import_users_data(&vec![input1, input2]);

		// assert
		assert!(res.is_ok());
		assert_eq!(state.len(), 2);
		assert!(state.contains_user_graph(&dsnp_user_id_1));
		assert!(state.contains_user_graph(&dsnp_user_id_2));
		assert!(!state.contains_user_graph(&(dsnp_user_id_2 + 1)));
	}

	#[test]
	fn api_import_user_data_with_invalid_page_content_should_fail() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id = get_schema_from(env.clone(), ConnectionType::Follow(PrivacyType::Public));
		let mut state = GraphState::new(env.clone());
		let dsnp_user_id = 1;
		let connections = vec![(2, 0), (3, 0), (4, 0), (5, 0)];
		let mut input = ImportBundleBuilder::new(env.clone(), dsnp_user_id, schema_id)
			.with_page(1, &connections, &vec![], 0)
			.build();
		let bad_page = PageData {
			content_hash: 1,
			content: vec![1, 2, 3], // invalid content
			page_id: 2,
		};
		input.pages.insert(1, bad_page);

		// act
		let res = state.import_users_data(&vec![input]);

		// assert
		assert!(res.is_err());
		assert_eq!(state.len(), 0);
		assert!(!state.contains_user_graph(&dsnp_user_id));
	}

	#[test]
	fn api_import_user_data_with_invalid_page_id_should_fail() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id = get_schema_from(env.clone(), ConnectionType::Follow(PrivacyType::Public));
		let mut state = GraphState::new(env.clone());
		let dsnp_user_id = 1;
		let connections = vec![(2, 0), (3, 0), (4, 0), (5, 0)];
		let mut input = ImportBundleBuilder::new(env.clone(), dsnp_user_id, schema_id)
			.with_page(1, &connections, &vec![], 0)
			.build();
		let mut bad_page = input.pages.remove(0);
		bad_page.page_id = env.get_config().max_page_id as PageId + 1; // invalid page id
		input.pages.push(bad_page);

		// act
		let res = state.import_users_data(&vec![input]);

		// assert
		assert!(res.is_err());
		assert_eq!(state.len(), 0);
		assert!(!state.contains_user_graph(&dsnp_user_id));
	}

	#[test]
	fn api_import_user_data_with_invalid_schema_id_should_fail() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id = 1000;
		let mut state = GraphState::new(env.clone());
		let dsnp_user_id = 1;
		let connections = vec![(2, 0), (3, 0), (4, 0), (5, 0)];
		let input = ImportBundleBuilder::new(env.clone(), dsnp_user_id, schema_id)
			.with_page(1, &connections, &vec![], 0)
			.build();

		// act
		let res = state.import_users_data(&vec![input]);

		// assert
		assert!(res.is_err());
	}

	#[test]
	fn api_import_user_data_with_invalid_serialized_public_key_should_fail() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id = get_schema_from(env.clone(), ConnectionType::Follow(PrivacyType::Public));
		let mut state = GraphState::new(env.clone());
		let dsnp_user_id = 1;
		let graph_key_pair = GraphKeyPair {
			key_type: GraphKeyType::X25519,
			secret_key: vec![],
			public_key: vec![0u8, 1u8], // invalid serialized public key
		};
		let connections = vec![(2, 0), (3, 0), (4, 0), (5, 0)];
		let input = ImportBundleBuilder::new(env.clone(), dsnp_user_id, schema_id)
			.with_page(1, &connections, &vec![], 0)
			.with_key_pairs(&vec![graph_key_pair])
			.build();

		// act
		let res = state.import_users_data(&vec![input]);

		// assert
		assert!(res.is_err());
	}

	#[test]
	fn api_import_user_data_with_maxed_out_capacity_should_fail() {
		// arrange
		let connection_type = ConnectionType::Follow(PrivacyType::Public);
		let schema_id = get_schema_from(Environment::Mainnet, connection_type);
		let env = Environment::Dev(
			ConfigBuilder::new()
				.with_schema(
					schema_id,
					SchemaConfig { connection_type, dsnp_version: DsnpVersion::Version1_0 },
				)
				.with_sdk_max_users_graph_size(0)
				.build(),
		);
		let mut state = GraphState::new(env.clone());
		let dsnp_user_id = 1;
		let connections = vec![(2, 0), (3, 0), (4, 0), (5, 0)];
		let input = ImportBundleBuilder::new(env.clone(), dsnp_user_id, schema_id)
			.with_page(1, &connections, &vec![], 0)
			.build();

		// act
		let res = state.import_users_data(&vec![input]);

		// assert
		assert!(res.is_err());
	}

	#[test]
	fn api_import_user_data_with_invalid_secret_key_should_fail() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id = get_schema_from(env.clone(), ConnectionType::Follow(PrivacyType::Private));
		let mut state = GraphState::new(env.clone());
		let graph_key_pair = GraphKeyPair {
			key_type: GraphKeyType::X25519,
			public_key: StackKeyPair::new().public_key.to_vec(),
			secret_key: vec![1, 2, 3, 4, 5, 6], // invalid secret key
		};
		let dsnp_user_id = 1;
		let connections = vec![(2, 0), (3, 0), (4, 0), (5, 0)];
		let input = ImportBundleBuilder::new(env.clone(), dsnp_user_id, schema_id)
			.with_page(1, &connections, &vec![], 0)
			.with_key_pairs(&vec![graph_key_pair])
			.build();

		// act
		let res = state.import_users_data(&vec![input]);

		// assert
		assert!(res.is_err());
	}

	#[test]
	fn api_import_user_data_with_incompatible_public_and_secret_key_should_fail() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id = get_schema_from(env.clone(), ConnectionType::Follow(PrivacyType::Private));
		let mut state = GraphState::new(env.clone());
		let graph_key_pair = GraphKeyPair {
			key_type: GraphKeyType::X25519,
			public_key: StackKeyPair::gen().public_key.to_vec(),
			secret_key: StackKeyPair::gen().secret_key.to_vec(),
		};
		let dsnp_user_id = 1;
		let connections = vec![(2, 0), (3, 0), (4, 0), (5, 0)];
		let input = ImportBundleBuilder::new(env.clone(), dsnp_user_id, schema_id)
			.with_page(1, &connections, &vec![], 0)
			.with_key_pairs(&vec![graph_key_pair])
			.build();

		// act
		let res = state.import_users_data(&vec![input]);

		// assert
		assert!(res.is_err());
	}

	#[test]
	fn api_import_user_data_should_import_graph_for_private_follow_successfully() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id = get_schema_from(env.clone(), ConnectionType::Follow(PrivacyType::Private));
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
		let res = state.import_users_data(&vec![input]);

		// assert
		assert!(res.is_ok());
		assert_eq!(state.len(), 1);
		assert!(state.contains_user_graph(&dsnp_user_id));
		assert!(!state.contains_user_graph(&(dsnp_user_id + 1)));
	}

	#[test]
	fn api_import_user_data_with_wrong_encryption_keys_should_fail() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id = get_schema_from(env.clone(), ConnectionType::Follow(PrivacyType::Private));
		let mut state = GraphState::new(env.clone());
		let key_pair_raw = StackKeyPair::gen();
		let resolved_key =
			ResolvedKeyPair { key_pair: KeyPairType::Version1_0(key_pair_raw.clone()), key_id: 2 };
		let key_pair_raw2 = StackKeyPair::gen();
		let keypair = GraphKeyPair {
			secret_key: key_pair_raw2.secret_key.to_vec(),
			public_key: key_pair_raw2.public_key.to_vec(),
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
		let res = state.import_users_data(&vec![input]);

		// assert
		assert!(res.is_err());
		assert!(!state.contains_user_graph(&dsnp_user_id));
	}

	#[test]
	fn api_import_user_data_with_no_resolved_keys_should_fail() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id = get_schema_from(env.clone(), ConnectionType::Follow(PrivacyType::Private));
		let mut state = GraphState::new(env.clone());
		let key_pair_raw = StackKeyPair::gen();
		let resolved_key =
			ResolvedKeyPair { key_pair: KeyPairType::Version1_0(key_pair_raw.clone()), key_id: 2 };
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
		let key_pair_raw2 = StackKeyPair::gen();
		let key_pair_2 = GraphKeyPair {
			secret_key: key_pair_raw2.secret_key.to_vec(),
			public_key: key_pair_raw2.public_key.to_vec(),
			key_type: GraphKeyType::X25519,
		};
		input.key_pairs = vec![key_pair_2];

		// act
		let res = state.import_users_data(&vec![input]);

		// assert
		assert!(res.is_err());
		assert!(!state.contains_user_graph(&dsnp_user_id));
	}

	#[test]
	fn api_import_user_data_should_import_graph_for_private_friendship_successfully() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id =
			get_schema_from(env.clone(), ConnectionType::Friendship(PrivacyType::Private));
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
		let connections: Vec<(DsnpUserId, u64)> = vec![(2, 0), (3, 0), (4, 0), (5, 0)];
		let prids: Vec<_> =
			connections.iter().map(|(id, _)| DsnpPrid::new(&id.to_le_bytes())).collect();
		let input = ImportBundleBuilder::new(env, dsnp_user_id, schema_id)
			.with_key_pairs(&vec![keypair])
			.with_encryption_key(resolved_key)
			.with_page(1, &connections, &prids, 0)
			.build();

		// act
		let res = state.import_users_data(&vec![input]);

		// assert
		assert!(res.is_ok());
		assert_eq!(state.len(), 1);
		assert!(state.contains_user_graph(&dsnp_user_id));
		assert!(!state.contains_user_graph(&(dsnp_user_id + 1)));
	}

	#[test]
	fn api_import_user_data_with_invalid_number_of_prids_for_private_friendship_should_fail() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id =
			get_schema_from(env.clone(), ConnectionType::Friendship(PrivacyType::Private));
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
		let connections: Vec<(DsnpUserId, u64)> = vec![(2, 0), (3, 0), (4, 0), (5, 0)];
		let prids: Vec<_> =
			connections.iter().map(|(id, _)| DsnpPrid::new(&id.to_le_bytes())).collect();
		let input = ImportBundleBuilder::new(env, dsnp_user_id, schema_id)
			.with_key_pairs(&vec![keypair])
			.with_encryption_key(resolved_key)
			.with_page(1, &connections, &prids[1..], 0)
			.build();

		// act
		let res = state.import_users_data(&vec![input]);

		// assert
		assert!(res.is_err());
		assert!(!state.contains_user_graph(&dsnp_user_id));
	}

	#[test]
	fn api_remove_user_graph_should_remove_user_successfully() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id = get_schema_from(env.clone(), ConnectionType::Follow(PrivacyType::Public));
		let mut state = GraphState::new(env.clone());
		let dsnp_user_id_1 = 1;
		let connections_1 = vec![(2, 0), (3, 0), (4, 0), (5, 0)];
		let input1 = ImportBundleBuilder::new(env.clone(), dsnp_user_id_1, schema_id)
			.with_page(1, &connections_1, &vec![], 0)
			.build();
		state.import_users_data(&vec![input1]).expect("Should import!");

		// act
		state.remove_user_graph(&dsnp_user_id_1);

		// assert
		assert_eq!(state.len(), 0);
		assert!(!state.contains_user_graph(&dsnp_user_id_1));
	}

	#[test]
	fn api_get_connections_for_user_graph_for_public_follow_should_return_all_connections() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id = get_schema_from(env.clone(), ConnectionType::Follow(PrivacyType::Public));
		let mut state = GraphState::new(env.clone());
		let dsnp_user_id_1 = 1;
		let connections_1 = vec![(2, 1), (3, 2), (4, 3), (5, 4)];
		let input1 = ImportBundleBuilder::new(env, dsnp_user_id_1, schema_id)
			.with_page(1, &connections_1, &vec![], 0)
			.build();
		state.import_users_data(&vec![input1]).expect("should import!");

		// act
		let res = state.get_connections_for_user_graph(&dsnp_user_id_1, &schema_id, false);

		// assert
		assert!(res.is_ok());
		let res_set: HashSet<_> = res.unwrap().iter().copied().collect();
		let mapped: HashSet<_> = connections_1
			.into_iter()
			.map(|(c, s)| DsnpGraphEdge { user_id: c, since: s })
			.collect();
		assert_eq!(res_set, mapped);
	}

	#[test]
	fn api_get_connections_for_user_graph_with_non_imported_user_should_fail() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id = get_schema_from(env.clone(), ConnectionType::Follow(PrivacyType::Public));
		let state = GraphState::new(env);
		let dsnp_user_id_1 = 1;

		// act
		let res = state.get_connections_for_user_graph(&dsnp_user_id_1, &schema_id, false);

		// assert
		assert!(res.is_err());
	}

	#[test]
	fn api_get_connections_without_keys_for_public_follow_should_return_all_connections() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id =
			get_schema_from(env.clone(), ConnectionType::Friendship(PrivacyType::Private));
		let mut state = GraphState::new(env.clone());
		let key_pair_raw = StackKeyPair::gen();
		let resolved_key =
			ResolvedKeyPair { key_pair: KeyPairType::Version1_0(key_pair_raw.clone()), key_id: 1 };
		let keypair = GraphKeyPair {
			secret_key: key_pair_raw.secret_key.to_vec(),
			public_key: key_pair_raw.public_key.to_vec(),
			key_type: GraphKeyType::X25519,
		};
		let dsnp_user_id_1 = 1;
		let connections_1: Vec<(DsnpUserId, u64)> = vec![(2, 1), (3, 2), (4, 3), (5, 4)];
		let prids: Vec<_> =
			connections_1.iter().map(|(id, _)| DsnpPrid::new(&id.to_le_bytes())).collect();
		let input1 = ImportBundleBuilder::new(env, dsnp_user_id_1, schema_id)
			.with_page(1, &connections_1, &prids, 0)
			.with_key_pairs(&vec![keypair])
			.with_encryption_key(resolved_key)
			.build();
		state.import_users_data(&vec![input1]).expect("should import!");

		// act
		let res = state.get_connections_without_keys();

		// assert
		assert!(res.is_ok());
		let res_set: HashSet<_> = res.unwrap().iter().copied().collect();
		let mapped: HashSet<_> = connections_1.into_iter().map(|(c, _)| c).collect();
		assert_eq!(res_set, mapped);
	}

	#[test]
	fn api_get_connections_without_keys_for_non_private_friendship_graph_should_be_empty() {
		// arrange
		for connection_type in vec![
			ConnectionType::Follow(PrivacyType::Public),
			ConnectionType::Follow(PrivacyType::Private),
			ConnectionType::Friendship(PrivacyType::Public),
		] {
			let env = Environment::Mainnet;
			let schema_id = get_schema_from(env.clone(), connection_type);
			let mut state = GraphState::new(env.clone());
			let key_pair_raw = StackKeyPair::gen();
			let resolved_key = ResolvedKeyPair {
				key_pair: KeyPairType::Version1_0(key_pair_raw.clone()),
				key_id: 1,
			};
			let keypair = GraphKeyPair {
				secret_key: key_pair_raw.secret_key.to_vec(),
				public_key: key_pair_raw.public_key.to_vec(),
				key_type: GraphKeyType::X25519,
			};
			let dsnp_user_id_1 = 1;
			let connections_1: Vec<(DsnpUserId, u64)> = vec![(2, 1), (3, 2), (4, 3), (5, 4)];
			let input1 = ImportBundleBuilder::new(env, dsnp_user_id_1, schema_id)
				.with_page(1, &connections_1, &vec![], 0)
				.with_key_pairs(&vec![keypair])
				.with_encryption_key(resolved_key)
				.build();
			state.import_users_data(&vec![input1]).expect("should import!");

			// act
			let res = state.get_connections_without_keys();

			// assert
			assert!(res.is_ok());
			assert_eq!(res.unwrap().len(), 0);
		}
	}

	#[test]
	fn api_get_one_sided_private_friendship_connections_for_public_follow_should_return_expected_connections(
	) {
		// arrange
		let env = Environment::Mainnet;
		let schema_id =
			get_schema_from(env.clone(), ConnectionType::Friendship(PrivacyType::Private));
		let mut state = GraphState::new(env.clone());
		let dsnp_user_id_1 = 1;
		// keys for dsnp user 1
		let key_pair_raw_1 = StackKeyPair::gen();
		let resolved_key_1 = ResolvedKeyPair {
			key_pair: KeyPairType::Version1_0(key_pair_raw_1.clone()),
			key_id: 0,
		};
		let keypair_1 = GraphKeyPair {
			secret_key: key_pair_raw_1.secret_key.to_vec(),
			public_key: key_pair_raw_1.public_key.to_vec(),
			key_type: GraphKeyType::X25519,
		};
		// keys for dsnp user 2
		let key_pair_raw_2 = StackKeyPair::gen();
		let resolved_key_2 = ResolvedKeyPair {
			key_pair: KeyPairType::Version1_0(key_pair_raw_2.clone()),
			key_id: 0,
		};
		let keypair_2 = GraphKeyPair {
			secret_key: key_pair_raw_2.secret_key.to_vec(),
			public_key: key_pair_raw_2.public_key.to_vec(),
			key_type: GraphKeyType::X25519,
		};
		// keys for dsnp user 3
		let key_pair_raw_3 = StackKeyPair::gen();
		let resolved_key_3 = ResolvedKeyPair {
			key_pair: KeyPairType::Version1_0(key_pair_raw_3.clone()),
			key_id: 0,
		};
		let keypair_3 = GraphKeyPair {
			secret_key: key_pair_raw_3.secret_key.to_vec(),
			public_key: key_pair_raw_3.public_key.to_vec(),
			key_type: GraphKeyType::X25519,
		};
		// --------- user 1 graph setup--------------
		let connections_1: Vec<(DsnpUserId, u64)> = vec![(2, 0), (3, 0)];
		let prids: Vec<_> = vec![
			DsnpPrid::create_prid(
				dsnp_user_id_1,
				2,
				&resolved_key_1.key_pair.clone().into(),
				&resolved_key_2.key_pair.borrow().into(),
			)
			.unwrap(),
			DsnpPrid::create_prid(
				dsnp_user_id_1,
				3,
				&resolved_key_1.key_pair.clone().into(),
				&resolved_key_3.key_pair.borrow().into(),
			)
			.unwrap(),
		];
		let input1 = ImportBundleBuilder::new(env.clone(), dsnp_user_id_1, schema_id)
			.with_page(1, &connections_1, &prids, 0)
			.with_key_pairs(&vec![keypair_1])
			.with_encryption_key(resolved_key_1.clone())
			.build();
		// --------- user 2 graph setup--------------
		let connections_2: Vec<(DsnpUserId, u64)> = vec![(3, 0)];
		let prids_2: Vec<_> = vec![DsnpPrid::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7])];
		let mut input2 = ImportBundleBuilder::new(env.clone(), 2, schema_id)
			.with_page(1, &connections_2, &prids_2, 0)
			.with_key_pairs(&vec![keypair_2])
			.build();
		input2.key_pairs = vec![];
		// --------- user 3 graph setup--------------
		let connections_3: Vec<(DsnpUserId, u64)> = vec![(dsnp_user_id_1, 0)];
		let prids_3: Vec<_> = vec![DsnpPrid::create_prid(
			3,
			dsnp_user_id_1,
			&resolved_key_1.key_pair.clone().into(),
			&resolved_key_3.key_pair.borrow().into(),
		)
		.unwrap()];
		let mut input3 = ImportBundleBuilder::new(env.clone(), 3, schema_id)
			.with_page(1, &connections_3, &prids_3, 0)
			.with_key_pairs(&vec![keypair_3])
			.build();
		input3.key_pairs = vec![];
		state.import_users_data(&vec![input1, input2, input3]).expect("should import!");

		// act
		let res = state.get_one_sided_private_friendship_connections(&dsnp_user_id_1);

		// assert
		assert!(res.is_ok());
		let res_set: HashSet<_> = res.unwrap().iter().copied().collect();
		let mapped: HashSet<_> = HashSet::from([DsnpGraphEdge { user_id: 2, since: 0 }]);
		assert_eq!(res_set, mapped);
	}

	#[test]
	fn api_apply_actions_should_work_as_expected_and_include_changes_in_pending() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id = get_schema_from(env.clone(), ConnectionType::Follow(PrivacyType::Public));
		let mut state = GraphState::new(env.clone());
		let dsnp_user_id_1 = 1;
		let connections_1 = vec![(2, 1), (3, 2), (4, 3), (5, 4)];
		let input1 = ImportBundleBuilder::new(env, dsnp_user_id_1, schema_id)
			.with_page(1, &connections_1, &vec![], 0)
			.build();
		state.import_users_data(&vec![input1]).expect("should import!");
		let actions = vec![
			Action::AddGraphKey {
				owner_dsnp_user_id: dsnp_user_id_1,
				new_public_key: StackKeyPair::gen().public_key.to_vec(),
			},
			Action::Connect {
				owner_dsnp_user_id: dsnp_user_id_1,
				connection: Connection { dsnp_user_id: 10, schema_id },
			},
			Action::Disconnect {
				owner_dsnp_user_id: dsnp_user_id_1,
				connection: Connection { dsnp_user_id: 3, schema_id },
			},
		];
		let expected_connections = vec![(2, 1), (10, 0), (4, 3), (5, 4)];

		// act
		let res = state.apply_actions(&actions);

		// assert
		assert!(res.is_ok());
		let connections = state
			.get_connections_for_user_graph(&dsnp_user_id_1, &schema_id, true)
			.expect("should work");
		let sorted_connections: HashSet<_> = connections.into_iter().map(|e| e.user_id).collect();
		let mapped: HashSet<_> = expected_connections.into_iter().map(|(c, _)| c).collect();
		assert_eq!(sorted_connections, mapped);
	}

	#[test]
	fn api_apply_actions_add_with_exising_connections_should_fail() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id = get_schema_from(env.clone(), ConnectionType::Follow(PrivacyType::Public));
		let mut state = GraphState::new(env.clone());
		let dsnp_user_id_1 = 1;
		let connections_1 = vec![(2, 1), (3, 2), (4, 3), (5, 4)];
		let input1 = ImportBundleBuilder::new(env, dsnp_user_id_1, schema_id)
			.with_page(1, &connections_1, &vec![], 0)
			.build();
		state.import_users_data(&vec![input1]).expect("should import!");
		let actions = vec![Action::Connect {
			owner_dsnp_user_id: dsnp_user_id_1,
			connection: Connection { dsnp_user_id: 5, schema_id },
		}];

		// act
		let res = state.apply_actions(&actions);

		// assert
		assert!(res.is_err());
	}

	#[test]
	fn api_apply_actions_remove_with_non_existing_connections_should_fail() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id = get_schema_from(env.clone(), ConnectionType::Follow(PrivacyType::Public));
		let mut state = GraphState::new(env.clone());
		let dsnp_user_id_1 = 1;
		let connections_1 = vec![(2, 1), (3, 2), (4, 3), (5, 4)];
		let input1 = ImportBundleBuilder::new(env, dsnp_user_id_1, schema_id)
			.with_page(1, &connections_1, &vec![], 0)
			.build();
		state.import_users_data(&vec![input1]).expect("should import!");
		let actions = vec![Action::Disconnect {
			owner_dsnp_user_id: dsnp_user_id_1,
			connection: Connection { dsnp_user_id: 10, schema_id },
		}];

		// act
		let res = state.apply_actions(&actions);

		// assert
		assert!(res.is_err());
	}

	#[test]
	fn api_apply_actions_with_failure_should_revert_all_the_changes() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id = get_schema_from(env.clone(), ConnectionType::Follow(PrivacyType::Public));
		let mut state = GraphState::new(env.clone());
		let dsnp_user_id_1 = 1;
		let connections_1 = vec![(2, 1), (3, 2), (4, 3), (5, 4)];
		let input1 = ImportBundleBuilder::new(env, dsnp_user_id_1, schema_id)
			.with_page(1, &connections_1, &vec![], 0)
			.build();
		state.import_users_data(&vec![input1]).expect("should import!");
		let actions = vec![
			Action::Connect {
				owner_dsnp_user_id: dsnp_user_id_1,
				connection: Connection { dsnp_user_id: 1000, schema_id },
			},
			Action::Disconnect {
				owner_dsnp_user_id: dsnp_user_id_1,
				connection: Connection { dsnp_user_id: 10, schema_id },
			},
		];

		// act
		let res = state.apply_actions(&actions);

		// assert
		assert!(res.is_err());
		let connections = state
			.get_connections_for_user_graph(&dsnp_user_id_1, &schema_id, true)
			.expect("should work");
		assert!(!connections.iter().any(|e| e.user_id == 1000));
	}

	#[test]
	fn api_apply_actions_with_duplicate_connection_for_user_fails() {
		// arrange
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

		// act
		assert!(state.apply_actions(&vec![action.clone()]).is_ok());
		assert!(state.apply_actions(&vec![action]).is_err());
	}

	#[test]
	fn api_apply_actions_with_remove_connection_for_user_twice_fails() {
		// arrange
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
	fn api_apply_actions_with_remove_connection_from_nonexistent_user_fails() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id = env
			.get_config()
			.get_schema_id_from_connection_type(ConnectionType::Follow(PrivacyType::Private))
			.expect("should exist");
		let mut state = GraphState::new(env);

		// act
		assert!(state
			.apply_actions(&vec![Action::Disconnect {
				owner_dsnp_user_id: 0,
				connection: Connection { dsnp_user_id: 1, schema_id }
			}])
			.is_err());
	}

	#[test]
	fn api_export_updates_for_public_graph_should_return_the_updated_pages_successfully() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id = get_schema_from(env.clone(), ConnectionType::Follow(PrivacyType::Public));
		let mut state = GraphState::new(env.clone());
		let dsnp_user_id_1 = 1;
		let connections_1 = vec![(2, 1), (3, 2), (4, 3), (5, 4)];
		let connections_2 = vec![(10, 1), (20, 2)];
		let connections_3 = vec![(100, 1)];
		let input1 = ImportBundleBuilder::new(env.clone(), dsnp_user_id_1, schema_id)
			.with_page(1, &connections_1, &vec![], 1)
			.with_page(2, &connections_2, &vec![], 2)
			.with_page(3, &connections_3, &vec![], 3)
			.build();
		state.import_users_data(&vec![input1.clone()]).expect("should import!");
		let new_public_key = StackKeyPair::gen().public_key.to_vec();
		let actions = vec![
			Action::AddGraphKey { owner_dsnp_user_id: dsnp_user_id_1, new_public_key },
			Action::Connect {
				owner_dsnp_user_id: dsnp_user_id_1,
				connection: Connection { dsnp_user_id: 6, schema_id },
			},
			Action::Disconnect {
				owner_dsnp_user_id: dsnp_user_id_1,
				connection: Connection { dsnp_user_id: 10, schema_id },
			},
			Action::Disconnect {
				owner_dsnp_user_id: dsnp_user_id_1,
				connection: Connection { dsnp_user_id: 100, schema_id },
			},
		];
		state.apply_actions(&actions).expect("Should apply actions!");
		let expected_connections = HashSet::<DsnpUserId>::from([2, 3, 4, 5, 6, 20]);

		// act
		let result = state.export_updates();

		// assert
		assert!(result.is_ok());
		let exports = result.unwrap();
		let mut state = GraphState::new(env);
		let input2 = ImportBundleBuilder::build_from(&input1, &exports);
		assert_eq!(input2.pages.len(), input1.pages.len() - 1);
		assert_eq!(input2.dsnp_keys.keys.len(), input1.dsnp_keys.keys.len() + 1);
		state.import_users_data(&vec![input2]).expect("should import input2");
		let new_connections: HashSet<DsnpUserId> = state
			.get_connections_for_user_graph(&dsnp_user_id_1, &schema_id, false)
			.unwrap()
			.iter()
			.map(|e| e.user_id)
			.collect();
		assert_eq!(new_connections, expected_connections);
	}

	#[test]
	fn api_export_updates_for_private_follow_graph_should_return_the_updated_pages_successfully() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id = get_schema_from(env.clone(), ConnectionType::Follow(PrivacyType::Private));
		let mut state = GraphState::new(env.clone());
		let dsnp_user_id_1 = 1;
		let connections_1 = vec![(2, 1), (3, 2), (4, 3), (5, 4)];
		let connections_2 = vec![(10, 1), (20, 2)];
		let connections_3 = vec![(100, 1)];
		let key_pair_raw = StackKeyPair::gen();
		let resolved_key =
			ResolvedKeyPair { key_pair: KeyPairType::Version1_0(key_pair_raw.clone()), key_id: 0 };
		let keypair = GraphKeyPair {
			secret_key: key_pair_raw.secret_key.to_vec(),
			public_key: key_pair_raw.public_key.to_vec(),
			key_type: GraphKeyType::X25519,
		};
		let input1 = ImportBundleBuilder::new(env.clone(), dsnp_user_id_1, schema_id)
			.with_page(1, &connections_1, &vec![], 1)
			.with_page(2, &connections_2, &vec![], 2)
			.with_page(3, &connections_3, &vec![], 3)
			.with_key_pairs(&vec![keypair])
			.with_encryption_key(resolved_key)
			.build();
		state.import_users_data(&vec![input1.clone()]).expect("should import!");
		let actions = vec![
			Action::Connect {
				owner_dsnp_user_id: dsnp_user_id_1,
				connection: Connection { dsnp_user_id: 6, schema_id },
			},
			Action::Disconnect {
				owner_dsnp_user_id: dsnp_user_id_1,
				connection: Connection { dsnp_user_id: 10, schema_id },
			},
			Action::Disconnect {
				owner_dsnp_user_id: dsnp_user_id_1,
				connection: Connection { dsnp_user_id: 100, schema_id },
			},
		];
		state.apply_actions(&actions).expect("Should apply actions!");
		let expected_connections = HashSet::<DsnpUserId>::from([2, 3, 4, 5, 6, 20]);

		// act
		let result = state.export_updates();

		// assert
		assert!(result.is_ok());
		let exports = result.unwrap();
		let mut state = GraphState::new(env);
		let input2 = ImportBundleBuilder::build_from(&input1, &exports);
		assert_eq!(input2.pages.len(), input1.pages.len() - 1);
		assert_eq!(input2.dsnp_keys.keys.len(), input1.dsnp_keys.keys.len());
		state.import_users_data(&vec![input2]).expect("should import input2");
		let new_connections: HashSet<DsnpUserId> = state
			.get_connections_for_user_graph(&dsnp_user_id_1, &schema_id, false)
			.unwrap()
			.iter()
			.map(|e| e.user_id)
			.collect();
		assert_eq!(new_connections, expected_connections);
	}

	#[test]
	fn api_get_public_keys_should_return_imported_public_keys() {
		// arrange
		let env = Environment::Mainnet;
		let schema_id = get_schema_from(env.clone(), ConnectionType::Follow(PrivacyType::Public));
		let mut state = GraphState::new(env.clone());
		let key_pair_raw = StackKeyPair::gen();
		let key_pair_raw_2 = StackKeyPair::gen();
		let keypair = GraphKeyPair {
			secret_key: key_pair_raw.secret_key.to_vec(),
			public_key: key_pair_raw.public_key.to_vec(),
			key_type: GraphKeyType::X25519,
		};
		let keypair_2 = GraphKeyPair {
			secret_key: key_pair_raw_2.secret_key.to_vec(),
			public_key: key_pair_raw_2.public_key.to_vec(),
			key_type: GraphKeyType::X25519,
		};
		let dsnp_user_id = 123;
		let connections = vec![(2, 0), (3, 0), (4, 0), (5, 0)];
		let input = ImportBundleBuilder::new(env, dsnp_user_id, schema_id)
			.with_key_pairs(&vec![keypair, keypair_2])
			.with_page(1, &connections, &vec![], 0)
			.build();
		state.import_users_data(&vec![input]).expect("Import should work!");

		// act
		let res = state.get_public_keys(&dsnp_user_id);

		// assert
		assert!(res.is_ok());
		let keys = res.unwrap();
		assert_eq!(
			keys,
			vec![
				DsnpPublicKey { key: key_pair_raw.public_key.to_vec(), key_id: Some(0) },
				DsnpPublicKey { key: key_pair_raw_2.public_key.to_vec(), key_id: Some(1) }
			]
		);
	}
}
