use crate::{
	dsnp::api_types::{ConnectionType, PrivacyType},
	frequency,
	frequency::{config::*, Frequency},
};
use pretty_assertions::assert_eq;
use std::collections::hash_map::HashMap;

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
