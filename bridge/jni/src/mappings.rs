use dsnp_graph_config::{
	Config as RustConfig, DsnpVersion as RustDsnpVersion, Environment as RustEnvironment,
	SchemaConfig as RustSchemaConfig, SchemaId,
};
use dsnp_graph_core::dsnp::api_types::{
	ConnectionType as RustConnectionType, PrivacyType as RustPrivacyType,
};
use dsnp_graph_sdk_common::proto_types::output as proto_output;
use jni::{objects::JByteArray, JNIEnv};
use protobuf::Message;
use std::collections::HashMap;

pub fn map_to_environment<'local>(
	env: &JNIEnv<'local>,
	environment: &JByteArray,
) -> Option<RustEnvironment> {
	let bytes = env.convert_byte_array(environment).ok()?;
	let env_proto = proto_output::Environment::parse_from_bytes(&bytes).ok()?;
	let result = match env_proto.environment_type.enum_value().ok()? {
		proto_output::EnvironmentType::MainNet => RustEnvironment::Mainnet,
		proto_output::EnvironmentType::Rococo => RustEnvironment::Rococo,
		proto_output::EnvironmentType::Dev => {
			let cfg = env_proto.config.into_option()?;
			let rust_cfg = map_config_to_rust(cfg)?;
			RustEnvironment::Dev(rust_cfg)
		},
	};
	Some(result)
}

fn map_config_to_rust(config: proto_output::Config) -> Option<RustConfig> {
	let mut dsnp_versions = vec![];
	for version in config.dsnp_versions.into_iter() {
		dsnp_versions.push(map_dsnp_version_to_rust(version.enum_value().ok()?)?);
	}

	let mut schema_map = HashMap::<SchemaId, RustSchemaConfig>::new();
	for (key, val) in config.schema_map.into_iter() {
		schema_map.insert(key.try_into().ok()?, map_schema_config_to_rust(val)?);
	}

	Some(RustConfig {
		max_graph_page_size_bytes: config.max_graph_page_size_bytes,
		sdk_max_stale_friendship_days: config.sdk_max_stale_friendship_days,
		max_page_id: config.max_page_id,
		sdk_max_users_graph_size: config.sdk_max_users_graph_size,
		max_key_page_size_bytes: config.max_key_page_size_bytes,
		dsnp_versions,
		schema_map,
	})
}

fn map_dsnp_version_to_rust(version: proto_output::DsnpVersion) -> Option<RustDsnpVersion> {
	let result = match version {
		proto_output::DsnpVersion::Version1_0 => RustDsnpVersion::Version1_0,
	};
	Some(result)
}

fn map_schema_config_to_rust(
	schema_config: proto_output::SchemaConfig,
) -> Option<RustSchemaConfig> {
	let version = schema_config.dsnp_version.enum_value().ok()?;
	let connection_type = schema_config.connection_type.enum_value().ok()?;

	let result = RustSchemaConfig {
		dsnp_version: map_dsnp_version_to_rust(version)?,
		connection_type: map_connection_type_to_rust(connection_type)?,
	};

	Some(result)
}

fn map_connection_type_to_rust(
	connection_type: proto_output::ConnectionType,
) -> Option<RustConnectionType> {
	Some(match connection_type {
		proto_output::ConnectionType::FollowPrivate =>
			RustConnectionType::Follow(RustPrivacyType::Private),
		proto_output::ConnectionType::FollowPublic =>
			RustConnectionType::Follow(RustPrivacyType::Public),
		proto_output::ConnectionType::FriendshipPrivate =>
			RustConnectionType::Friendship(RustPrivacyType::Private),
		proto_output::ConnectionType::FriendshipPublic =>
			RustConnectionType::Friendship(RustPrivacyType::Public),
	})
}
