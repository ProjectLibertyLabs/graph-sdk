use apache_avro::Schema;
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};
use serde_with::serde_as;
use std::collections::hash_map::HashMap;

pub const MAINNET_CONFIG: &str = include_str!("../resources/configs/frequency.json");
pub const ROCOCO_CONFIG: &str = include_str!("../resources/configs/frequency-rococo.json");

pub type SchemaId = u16;

lazy_static! {
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
	pub schema_map: HashMap<ConnectionType, SchemaId>,
}

impl TryFrom<&str> for Config {
	type Error = serde_json::Error;
	fn try_from(s: &str) -> Result<Self, Self::Error> {
		serde_json::from_str(s)
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
		let parsed_config: Config = MAINNET_CONFIG.try_into()?;
		let config = Config {
			max_graph_page_size_bytes: 2048,
			max_page_id: 16,
			max_key_page_size_bytes: 1024,
			schema_map: HashMap::from([
				(ConnectionType::Follow(PrivacyType::Public), 1),
				(ConnectionType::Follow(PrivacyType::Private), 2),
				(ConnectionType::Friendship(PrivacyType::Public), 3),
				(ConnectionType::Friendship(PrivacyType::Private), 4),
			]),
		};

		assert_eq!(parsed_config, config);
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
}
