//! Config module contains all the settings and different environments that is supported by the
//! Graph SDk
//!
pub mod builder;
pub mod errors;
use crate::errors::DsnpGraphResult;
use apache_avro::Schema;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::{
	collections::hash_map::HashMap,
	fmt::{Display, Formatter},
};

/// SchemaId type
pub type SchemaId = u16;
/// DsnpUserId type
pub type DsnpUserId = u64;
/// PageId type
pub type PageId = u16;

/// a common trait to allow checks for api input types
pub trait InputValidation {
	fn validate(&self) -> DsnpGraphResult<()>;
}

lazy_static! {
	/// Schema for public key
	pub static ref PUBLIC_KEY_SCHEMA: Schema =
		Schema::parse_str(include_str!("../resources/schemas/public_key_schema.json")).unwrap();
	/// Schema for public graph chunk
	pub static ref PUBLIC_GRAPH_CHUNK_SCHEMA: Schema =
		Schema::parse_str(include_str!("../resources/schemas/user_public_graph_chunk.json"))
			.unwrap();
	/// Schema for public graph
	pub static ref PUBLIC_GRAPH_SCHEMA: Schema =
		Schema::parse_str(include_str!("../resources/schemas/public_graph.json")).unwrap();
	/// Schema for private graph chunk
	pub static ref PRIVATE_GRAPH_CHUNK_SCHEMA: Schema =
		Schema::parse_str(include_str!("../resources/schemas/user_private_graph_chunk.json"))
			.unwrap();

	/// Mainnet `Config`
	pub static ref MAINNET_CONFIG: Config = include_str!("../resources/configs/frequency.json")
		.try_into().unwrap();
	/// Rococo `Config`
	pub static ref ROCOCO_CONFIG: Config = include_str!("../resources/configs/frequency-rococo.json")
		.try_into().unwrap();
}

/// Privacy Type of the graph
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Ord, Eq, PartialOrd, Debug, Hash, Serialize, Deserialize)]
#[serde(tag = "privacyType")]
pub enum PrivacyType {
	/// publicly accessible graph
	#[serde(rename = "public")]
	Public,

	/// only accessible to owner of the graph and whoever the encryption keys have been shared with
	#[serde(rename = "private")]
	Private,
}

/// Different connection type in social graph
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Ord, Eq, PartialOrd, Debug, Hash, Serialize, Deserialize)]
#[serde(tag = "connectionType")]
pub enum ConnectionType {
	/// Follow is a one-way connection type, which means it is only stored in follower side
	#[serde(rename = "follow")]
	Follow(PrivacyType),

	/// Friendship is two-way connection type, which means it is stored in both sides and each
	/// side can revoke the connection for both sides
	#[serde(rename = "friendship")]
	Friendship(PrivacyType),
}

impl Display for ConnectionType {
	fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), std::fmt::Error> {
		use ConnectionType::*;
		write!(
			f,
			"{}",
			match self {
				Follow(PrivacyType::Public) => "Follow(Public)",
				Follow(PrivacyType::Private) => "Follow(Private)",
				Friendship(PrivacyType::Public) => "Friendship(Public)",
				Friendship(PrivacyType::Private) => "Friendship(Private)",
			}
		)
	}
}

impl ConnectionType {
	pub const fn privacy_type(&self) -> PrivacyType {
		match self {
			Self::Follow(privacy) | Self::Friendship(privacy) => *privacy,
		}
	}
}

/// a list of all supported Graphs and connections types
pub const ALL_CONNECTION_TYPES: [ConnectionType; 4] = [
	ConnectionType::Follow(PrivacyType::Public),
	ConnectionType::Friendship(PrivacyType::Public),
	ConnectionType::Follow(PrivacyType::Private),
	ConnectionType::Friendship(PrivacyType::Private),
];

/// Graph Key type
#[repr(C)]
#[derive(Debug, Clone, Eq, PartialEq)]
pub enum GraphKeyType {
	X25519 = 0,
}

/// Different environments supported by graph sdk
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Environment {
	Mainnet,
	Rococo,
	Dev(Config),
}

impl Environment {
	/// Returns the config for the environment
	pub fn get_config(&self) -> &Config {
		match self {
			Environment::Mainnet => &MAINNET_CONFIG,
			Environment::Rococo => &ROCOCO_CONFIG,
			Environment::Dev(cfg) => &cfg,
		}
	}
}

/// Supported Dsnp Versions
#[repr(C)]
#[derive(Clone, Copy, PartialEq, Ord, Eq, PartialOrd, Debug, Hash, Serialize, Deserialize)]
pub enum DsnpVersion {
	#[serde(rename = "1.0")]
	Version1_0,
}

/// Schema config
/// This is used to map schema id to dsnp version and connection type
#[repr(C)]
#[derive(Clone, PartialEq, Ord, Eq, PartialOrd, Debug, Hash, Serialize, Deserialize)]
pub struct SchemaConfig {
	pub dsnp_version: DsnpVersion,
	pub connection_type: ConnectionType,
}

/// Config
/// This is used to configure the graph state
#[serde_as]
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Config {
	/// Maximum number of users in the graph
	#[serde(rename = "sdkMaxUsersGraphSize")]
	pub sdk_max_users_graph_size: u32,

	/// Maximum number of days a friendship can be stale before it is removed
	#[serde(rename = "sdkMaxStaleFriendshipDays")]
	pub sdk_max_stale_friendship_days: u32,

	/// Maximum size of a graph page in bytes
	#[serde(rename = "maxGraphPageSizeBytes")]
	pub max_graph_page_size_bytes: u32,

	/// Maximum page id
	#[serde(rename = "maxPageId")]
	pub max_page_id: u32,

	/// Maximum size of a key page in bytes
	#[serde(rename = "maxKeyPageSizeBytes")]
	pub max_key_page_size_bytes: u32,

	/// Schema map
	#[serde(rename = "schemaMap")]
	#[serde_as(as = "Vec<(_, _)>")]
	pub schema_map: HashMap<SchemaId, SchemaConfig>,

	/// DSNP versions
	#[serde(rename = "dsnpVersions")]
	pub dsnp_versions: Vec<DsnpVersion>,
}

impl TryFrom<&str> for Config {
	type Error = serde_json::Error;
	fn try_from(s: &str) -> Result<Self, Self::Error> {
		log_err!(serde_json::from_str(s))
	}
}

impl Config {
	/// Returns the DSNP version for the given schema id
	pub fn get_dsnp_version_from_schema_id(&self, schema_id: SchemaId) -> Option<DsnpVersion> {
		if let Some(schema_config) = self.schema_map.get(&schema_id) {
			return Some(schema_config.dsnp_version)
		}
		log::warn!("no schema config found for schema ID {}", schema_id);
		None
	}

	/// Returns the connection type for the given schema id
	pub fn get_connection_type_from_schema_id(
		&self,
		schema_id: SchemaId,
	) -> Option<ConnectionType> {
		if let Some(schema_config) = self.schema_map.get(&schema_id) {
			return Some(schema_config.connection_type)
		}
		log::warn!("no schema config found for schema ID {}", schema_id);
		None
	}

	/// Returns the schema id for the given DSNP version and connection type
	pub fn get_schema_id_from_connection_type(
		&self,
		connection_type: ConnectionType,
	) -> Option<SchemaId> {
		match self
			.schema_map
			.iter()
			.filter_map(|(k, v)| {
				if v.connection_type == connection_type {
					return Some(*k)
				}
				None
			})
			.next()
		{
			Some(id) => Some(id),
			None => {
				log::warn!("no schema id found for connection type {}", connection_type);
				None
			},
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use test_log::test;

	#[test]
	fn connection_type_privacy_getter() {
		assert_eq!(ConnectionType::Follow(PrivacyType::Public).privacy_type(), PrivacyType::Public);
		assert_eq!(
			ConnectionType::Follow(PrivacyType::Private).privacy_type(),
			PrivacyType::Private
		);
		assert_eq!(
			ConnectionType::Friendship(PrivacyType::Public).privacy_type(),
			PrivacyType::Public
		);
		assert_eq!(
			ConnectionType::Friendship(PrivacyType::Private).privacy_type(),
			PrivacyType::Private
		);
	}

	#[test]
	fn config_import_success() -> Result<(), serde_json::Error> {
		let expected_config = Config {
			sdk_max_users_graph_size: 1000,
			sdk_max_stale_friendship_days: 90,
			max_graph_page_size_bytes: 1024,
			max_page_id: 16,
			max_key_page_size_bytes: 65536,
			dsnp_versions: vec![DsnpVersion::Version1_0],
			schema_map: HashMap::from([
				(
					1,
					SchemaConfig {
						dsnp_version: DsnpVersion::Version1_0,
						connection_type: ConnectionType::Follow(PrivacyType::Public),
					},
				),
				(
					2,
					SchemaConfig {
						dsnp_version: DsnpVersion::Version1_0,
						connection_type: ConnectionType::Follow(PrivacyType::Private),
					},
				),
				(
					3,
					SchemaConfig {
						dsnp_version: DsnpVersion::Version1_0,
						connection_type: ConnectionType::Friendship(PrivacyType::Public),
					},
				),
				(
					4,
					SchemaConfig {
						dsnp_version: DsnpVersion::Version1_0,
						connection_type: ConnectionType::Friendship(PrivacyType::Private),
					},
				),
			]),
		};

		assert_eq!(MAINNET_CONFIG.clone(), expected_config);
		Ok(())
	}

	#[test]
	fn config_import_failure() {
		assert!(<Config as TryFrom<&str>>::try_from("bad json").is_err());
	}

	#[test]
	fn lazy_static_schemas_are_valid() -> Result<(), apache_avro::Error> {
		let _ = PUBLIC_GRAPH_CHUNK_SCHEMA;
		let _ = PUBLIC_GRAPH_SCHEMA;
		let _ = PUBLIC_KEY_SCHEMA;
		let _ = PRIVATE_GRAPH_CHUNK_SCHEMA;
		Ok(())
	}

	#[test]
	fn lazy_static_configs_are_valid() -> Result<(), apache_avro::Error> {
		let _ = MAINNET_CONFIG;
		let _ = ROCOCO_CONFIG;
		Ok(())
	}
}
