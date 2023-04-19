mod builder;

use apache_avro::Schema;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::hash_map::HashMap;

pub type SchemaId = u16;

lazy_static! {
	// Schemas
	pub static ref PUBLIC_KEY_SCHEMA: Schema =
		Schema::parse_str(include_str!("../resources/schemas/public_key_schema.json")).unwrap();
	pub static ref PUBLIC_GRAPH_CHUNK_SCHEMA: Schema =
		Schema::parse_str(include_str!("../resources/schemas/user_public_graph_chunk.json"))
			.unwrap();
	pub static ref PUBLIC_GRAPH_SCHEMA: Schema =
		Schema::parse_str(include_str!("../resources/schemas/public_graph.json")).unwrap();
	pub static ref PRIVATE_GRAPH_CHUNK_SCHEMA: Schema =
		Schema::parse_str(include_str!("../resources/schemas/user_private_graph_chunk.json"))
			.unwrap();

	// Configurations
	pub static ref MAINNET_CONFIG: Config = include_str!("../resources/configs/frequency.json")
		.try_into().unwrap();
	pub static ref ROCOCO_CONFIG: Config = include_str!("../resources/configs/frequency-rococo.json")
		.try_into().unwrap();
}

/// Privacy Type of the graph
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

impl ConnectionType {
	pub const fn privacy_type(&self) -> PrivacyType {
		match self {
			Self::Follow(privacy) | Self::Friendship(privacy) => *privacy,
		}
	}
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Environment {
	Mainnet,
	Rococo,
	Dev(Config),
}

impl Environment {
	pub fn get_config(&self) -> &Config {
		match self {
			Environment::Mainnet => &MAINNET_CONFIG,
			Environment::Rococo => &ROCOCO_CONFIG,
			Environment::Dev(cfg) => &cfg,
		}
	}
}

#[derive(Clone, PartialEq, Ord, Eq, PartialOrd, Debug, Hash, Serialize, Deserialize)]
pub struct SchemaConfig {
	pub dsnp_version: String,
	pub connection_type: ConnectionType,
}

#[derive(Clone, Copy, PartialEq, Ord, Eq, PartialOrd, Debug, Hash, Serialize, Deserialize)]
pub enum EncryptionAlgorithm {
	#[serde(rename = "xSalsa20Poly1305")]
	XSalsa20Poly1305,
}

#[derive(Clone, Copy, PartialEq, Ord, Eq, PartialOrd, Debug, Hash, Serialize, Deserialize)]
pub enum KeyType {
	#[serde(rename = "x25519")]
	X25519,
}

#[derive(Clone, PartialEq, Ord, Eq, PartialOrd, Debug, Hash, Serialize, Deserialize)]
pub struct DsnpVersionConfig {
	#[serde(rename = "encryptionAlgo")]
	pub encryption_algo: EncryptionAlgorithm,

	#[serde(rename = "keyType")]
	pub key_type: KeyType,
}

#[serde_as]
#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Config {
	#[serde(rename = "maxGraphPageSizeBytes")]
	pub max_graph_page_size_bytes: u32,

	#[serde(rename = "maxPageId")]
	pub max_page_id: u32,

	#[serde(rename = "maxKeyPageSizeBytes")]
	pub max_key_page_size_bytes: u32,

	#[serde(rename = "schemaMap")]
	#[serde_as(as = "Vec<(_, _)>")]
	pub schema_map: HashMap<SchemaId, SchemaConfig>,

	#[serde(rename = "dsnpVersionMap")]
	#[serde_as(as = "Vec<(_, _)>")]
	pub dsnp_version_map: HashMap<String, DsnpVersionConfig>,
}

impl TryFrom<&str> for Config {
	type Error = serde_json::Error;
	fn try_from(s: &str) -> Result<Self, Self::Error> {
		serde_json::from_str(s)
	}
}

impl Config {
	pub fn get_dsnp_config_from_schema_id(
		&self,
		schema_id: SchemaId,
	) -> Option<(String, DsnpVersionConfig)> {
		if let Some(schema_config) = self.schema_map.get(&schema_id) {
			if let Some(dsnp_config) = self.dsnp_version_map.get(&schema_config.dsnp_version) {
				return Some((schema_config.dsnp_version.clone(), dsnp_config.clone()))
			}
		}
		None
	}

	pub fn get_connection_type_from_schema_id(
		&self,
		schema_id: SchemaId,
	) -> Option<ConnectionType> {
		if let Some(schema_config) = self.schema_map.get(&schema_id) {
			return Some(schema_config.connection_type)
		}
		None
	}

	pub fn get_schema_id_from_connection_type(
		&self,
		connection_type: ConnectionType,
	) -> Option<SchemaId> {
		self.schema_map
			.iter()
			.filter_map(|(k, v)| {
				if v.connection_type == connection_type {
					return Some(*k)
				}
				None
			})
			.next()
	}
}

#[cfg(test)]
mod test {
	use super::*;

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
			max_graph_page_size_bytes: 1024,
			max_page_id: 16,
			max_key_page_size_bytes: 65536,
			dsnp_version_map: HashMap::from([(
				"1.0".to_string(),
				DsnpVersionConfig {
					encryption_algo: EncryptionAlgorithm::XSalsa20Poly1305,
					key_type: KeyType::X25519,
				},
			)]),
			schema_map: HashMap::from([
				(
					1,
					SchemaConfig {
						dsnp_version: "1.0".to_string(),
						connection_type: ConnectionType::Follow(PrivacyType::Public),
					},
				),
				(
					2,
					SchemaConfig {
						dsnp_version: "1.0".to_string(),
						connection_type: ConnectionType::Follow(PrivacyType::Private),
					},
				),
				(
					3,
					SchemaConfig {
						dsnp_version: "1.0".to_string(),
						connection_type: ConnectionType::Friendship(PrivacyType::Public),
					},
				),
				(
					4,
					SchemaConfig {
						dsnp_version: "1.0".to_string(),
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
