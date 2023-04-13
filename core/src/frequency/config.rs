#![allow(dead_code)] // todo: remove
use crate::dsnp::api_types::{Config, ConnectionType, PrivacyType, SchemaId};
use std::collections::hash_map::HashMap;

pub struct ConfigMain;
impl Config for ConfigMain {
	// todo: update with correct values once mainnet has Graph schemas added
	fn schema_for_connection_type(&self, connection_type: ConnectionType) -> SchemaId {
		match connection_type {
			// todo: replace with real mainnet values once schemas have been created
			ConnectionType::Follow(PrivacyType::Public) => 3,
			ConnectionType::Follow(PrivacyType::Private) => 4,
			ConnectionType::Friendship(PrivacyType::Public) => 5,
			ConnectionType::Friendship(PrivacyType::Private) => 6,
		}
	}
}

pub struct ConfigRococo;
impl Config for ConfigRococo {
	// todo: update with correct values once Rococo has Graph schemas added
	fn schema_for_connection_type(&self, connection_type: ConnectionType) -> SchemaId {
		match connection_type {
			// todo: replace with real rococo values once schemas have been created
			ConnectionType::Follow(PrivacyType::Public) => 3,
			ConnectionType::Follow(PrivacyType::Private) => 4,
			ConnectionType::Friendship(PrivacyType::Public) => 5,
			ConnectionType::Friendship(PrivacyType::Private) => 6,
		}
	}
}

pub struct ConfigDev {
	schema_id_map: HashMap<ConnectionType, SchemaId>,
}

pub const PUBLIC_FOLLOW_SCHEMAID_ENV: &str = "FREQUENCY_PUBLIC_FOLLOW_SCHEMAID";
pub const PRIVATE_FOLLOW_SCHEMAID_ENV: &str = "FREQUENCY_PRIVATE_FOLLOW_SCHEMAID";
pub const PUBLIC_FRIEND_SCHEMAID_ENV: &str = "FREQUENCY_PUBLIC_FRIEND_SCHEMAID";
pub const PRIVATE_FRIEND_SCHEMAID_ENV: &str = "FREQUENCY_PRIVATE_FRIEND_SCHEMAID";

impl ConfigDev {
	fn env_var_to_connection_type(s: &str) -> ConnectionType {
		match s {
			PUBLIC_FOLLOW_SCHEMAID_ENV => ConnectionType::Follow(PrivacyType::Public),
			PRIVATE_FOLLOW_SCHEMAID_ENV => ConnectionType::Follow(PrivacyType::Private),
			PUBLIC_FRIEND_SCHEMAID_ENV => ConnectionType::Friendship(PrivacyType::Public),
			PRIVATE_FRIEND_SCHEMAID_ENV => ConnectionType::Friendship(PrivacyType::Private),
			_ => panic!("Unknown connection type string {}", s),
		}
	}

	pub fn new(config: Option<HashMap<ConnectionType, SchemaId>>) -> Self {
		let freq = match config {
			Some(schema_id_map) => Self { schema_id_map },
			None => {
				let env_vars = vec![
					PUBLIC_FOLLOW_SCHEMAID_ENV,
					PRIVATE_FOLLOW_SCHEMAID_ENV,
					PUBLIC_FRIEND_SCHEMAID_ENV,
					PRIVATE_FRIEND_SCHEMAID_ENV,
				];
				let mut schema_id_map = HashMap::<ConnectionType, SchemaId>::new();
				for e in env_vars {
					schema_id_map.insert(
						Self::env_var_to_connection_type(e),
						std::env::var(e)
							.unwrap_or_default()
							.as_str()
							.parse::<SchemaId>()
							.unwrap_or_default(),
					);
				}
				Self { schema_id_map }
			},
		};

		let connection_types: Vec<ConnectionType> = vec![
			ConnectionType::Follow(PrivacyType::Public),
			ConnectionType::Follow(PrivacyType::Private),
			ConnectionType::Friendship(PrivacyType::Public),
			ConnectionType::Friendship(PrivacyType::Private),
		];

		if !connection_types.iter().all(|c| freq.schema_id_map.contains_key(c)) {
			panic!("Incomplete Frequency development config");
		}

		freq
	}
}

impl Config for ConfigDev {
	fn schema_for_connection_type(&self, connection_type: ConnectionType) -> SchemaId {
		*self
			.schema_id_map
			.get(&connection_type)
			.expect("Incomplete configuration encountered")
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::{frequency, frequency::Frequency};

	#[test]
	fn mainnet_config_is_correct() {
		let freq = frequency!(ConfigMain);
		assert_eq!(
			freq.config
				.schema_for_connection_type(ConnectionType::Follow(PrivacyType::Public)),
			3
		);
		assert_eq!(
			freq.config
				.schema_for_connection_type(ConnectionType::Follow(PrivacyType::Private)),
			4
		);
		assert_eq!(
			freq.config
				.schema_for_connection_type(ConnectionType::Friendship(PrivacyType::Public)),
			5
		);
		assert_eq!(
			freq.config
				.schema_for_connection_type(ConnectionType::Friendship(PrivacyType::Private)),
			6
		);
	}

	#[test]
	fn rococo_config_is_correct() {
		let freq = frequency!(ConfigRococo);
		assert_eq!(
			freq.config
				.schema_for_connection_type(ConnectionType::Follow(PrivacyType::Public)),
			3
		);
		assert_eq!(
			freq.config
				.schema_for_connection_type(ConnectionType::Follow(PrivacyType::Private)),
			4
		);
		assert_eq!(
			freq.config
				.schema_for_connection_type(ConnectionType::Friendship(PrivacyType::Public)),
			5
		);
		assert_eq!(
			freq.config
				.schema_for_connection_type(ConnectionType::Friendship(PrivacyType::Private)),
			6
		);
	}

	#[test]
	fn dev_config_from_env_is_correct() {
		let mut curr_id = 100;
		let env_vars = vec![
			PUBLIC_FOLLOW_SCHEMAID_ENV,
			PRIVATE_FOLLOW_SCHEMAID_ENV,
			PUBLIC_FRIEND_SCHEMAID_ENV,
			PRIVATE_FRIEND_SCHEMAID_ENV,
		];
		// let mut schema_id_map = HashMap::<ConnectionType, SchemaId>::new();
		env_vars.iter().for_each(|e| {
			std::env::set_var(e, curr_id.to_string());
			curr_id += 1;
		});
		let freq = frequency!(ConfigDev);
		assert_eq!(
			freq.config
				.schema_for_connection_type(ConnectionType::Follow(PrivacyType::Public)),
			100
		);
		assert_eq!(
			freq.config
				.schema_for_connection_type(ConnectionType::Follow(PrivacyType::Private)),
			101
		);
		assert_eq!(
			freq.config
				.schema_for_connection_type(ConnectionType::Friendship(PrivacyType::Public)),
			102
		);
		assert_eq!(
			freq.config
				.schema_for_connection_type(ConnectionType::Friendship(PrivacyType::Private)),
			103
		);
	}

	#[test]
	fn dev_config_from_map_is_correct() {
		let freq = frequency!(ConfigDev { [
		(ConnectionType::Follow(PrivacyType::Public), 100),
		(ConnectionType::Follow(PrivacyType::Private), 101),
		(ConnectionType::Friendship(PrivacyType::Public), 102),
		(ConnectionType::Friendship(PrivacyType::Private), 103),
	] });
		assert_eq!(
			freq.config
				.schema_for_connection_type(ConnectionType::Follow(PrivacyType::Public)),
			100
		);
		assert_eq!(
			freq.config
				.schema_for_connection_type(ConnectionType::Follow(PrivacyType::Private)),
			101
		);
		assert_eq!(
			freq.config
				.schema_for_connection_type(ConnectionType::Friendship(PrivacyType::Public)),
			102
		);
		assert_eq!(
			freq.config
				.schema_for_connection_type(ConnectionType::Friendship(PrivacyType::Private)),
			103
		);
	}

	#[test]
	#[should_panic]
	fn dev_config_from_incomplete_map_panics() {
		let _ = frequency!(ConfigDev { [
		(ConnectionType::Follow(PrivacyType::Public), 100),
		(ConnectionType::Follow(PrivacyType::Private), 101),
		(ConnectionType::Friendship(PrivacyType::Public), 102),
	] });
	}
}
