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
		api_types::{GraphKeyPair, ResolvedKeyPair},
		dsnp_configs::KeyPairType,
	};

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
	fn api_import_user_data_should_for_public_follow_should_import_graph_successfully() {
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
}
