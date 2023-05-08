use dsnp_graph_config::{DsnpVersion, GraphKeyType, SchemaConfig, SchemaId};
use dsnp_graph_core::dsnp::{
	api_types::{PageHash, PageId},
	dsnp_types::{DsnpGraphEdge, DsnpUserId},
};
use libc::size_t;

/// `dsnp_graph_core::dsnp::api_types::KeyData` wrapper
#[repr(C)]
pub struct KeyData {
	pub index: u16,
	pub content: *mut u8,
	pub content_len: size_t,
}

/// `dsnp_graph_core::dsnp::api_types::GraphKeyPair` wrapper
#[repr(C)]
pub struct GraphKeyPair {
	/// key pair type
	pub key_type: GraphKeyType,

	/// public key raw
	pub public_key: *const u8,

	/// length of the public key
	pub public_key_len: size_t,

	/// secret key raw
	pub secret_key: *const u8,

	/// length of the secret key
	pub secret_key_len: size_t,
}

// `dsnp_graph_core::dsnp::api_types::PageData` wrapper
#[repr(C)]
pub struct PageData {
	// Id of the page
	pub page_id: PageId,

	// raw content of page data
	pub content: *mut u8,
	pub content_len: size_t,

	// hash value of content
	pub content_hash: PageHash,
}

// `dsnp_graph_core::dsnp::api_types::DsnpKeys` wrapper
#[repr(C)]
pub struct DsnpKeys {
	pub dsnp_user_id: DsnpUserId,
	pub keys_hash: PageHash,
	pub keys: *mut KeyData,
	pub keys_len: size_t,
}

// `dsnp_graph_core::dsnp::api_types::ImportBundle` wrapper
#[repr(C)]
pub struct ImportBundle {
	/// graph owner dsnp user id
	pub dsnp_user_id: DsnpUserId,

	/// Schema id of imported data
	pub schema_id: SchemaId,

	/// key pairs associated with this graph which is used for encryption and PRI generation
	pub key_pairs: *mut GraphKeyPair,
	pub key_pairs_len: size_t,

	/// published dsnp keys associated with this dsnp user
	pub dsnp_keys: DsnpKeys,

	/// Page data containing the social graph retrieved from chain
	pub pages: *mut PageData,
	pub pages_len: size_t,
}

// `dsnp_graph_core::dsnp::api_types::Update::PersistPage` wrapper
#[repr(C)]
pub struct PersistPage {
	/// owner of the social graph
	pub owner_dsnp_user_id: DsnpUserId,

	/// Schema id of imported data
	pub schema_id: SchemaId,

	/// page id associated with changed page
	pub page_id: PageId,

	/// previous hash value is used to avoid updating a stale state
	pub prev_hash: PageHash,

	/// social graph page data
	pub payload: *mut u8,
	pub payload_len: size_t,
}

// `dsnp_graph_core::dsnp::api_types::Update::DeletePage` wrapper
#[repr(C)]
pub struct DeletePage {
	/// owner of the social graph
	pub owner_dsnp_user_id: DsnpUserId,

	/// Schema id of removed data
	pub schema_id: SchemaId,

	/// page id associated with changed page
	pub page_id: PageId,

	/// previous hash value is used to avoid updating a stale state
	pub prev_hash: PageHash,
}

// `dsnp_graph_core::dsnp::api_types::Update::AddKey` wrapper
#[repr(C)]
pub struct AddKey {
	/// owner of the social graph
	pub owner_dsnp_user_id: DsnpUserId,

	/// previous hash value is used to avoid updating a stale state
	pub prev_hash: PageHash,

	/// social graph page data
	pub payload: *mut u8,
	pub payload_len: size_t,
}

// `dsnp_graph_core::dsnp::api_types::Update` wrapper
#[repr(C)]
pub enum Update {
	Persist(PersistPage),
	Delete(DeletePage),
	Add(AddKey),
}

// SchemaConfig tuple wrapper
#[repr(C)]
pub struct SchemaConfigTuple {
	pub schema_id: SchemaId,
	pub schema_config: SchemaConfig,
}

// `dsnp_graph_config::Config` wrapper
#[repr(C)]
pub struct Config {
	pub sdk_max_users_graph_size: u32,
	pub sdk_max_stale_friendship_days: u32,
	pub max_graph_page_size_bytes: u32,
	pub max_page_id: u32,
	pub max_key_page_size_bytes: u32,
	pub schema_map_len: size_t,
	pub schema_map: *mut SchemaConfigTuple,
	pub dsnp_versions_len: size_t,
	pub dsnp_versions: *mut DsnpVersion,
}

// EnvironmentType for `Config`
#[repr(C)]
pub enum EnvironmentType {
	Mainnet,
	Rococo,
	Dev,
}

// `dsnp_graph_config::Environment` wrapper
#[repr(C)]
pub struct Environment {
	pub environment_type: EnvironmentType,
	pub config: Config, // This field will only be used when environment_type is Dev.
}

// Output type for GraphConnection list
#[repr(C)]
pub struct GraphConnections {
	pub connections: *mut DsnpGraphEdge,
	pub connections_len: usize,
}

// Output type for GraphConnectionsWithoutKeys list
#[repr(C)]
pub struct GraphConnectionsWithoutKeys {
	pub connections: *mut DsnpUserId,
	pub connections_len: usize,
}

// Output type for GraphUpdates list
#[repr(C)]
pub struct GraphUpdates {
	pub updates: *mut Update,
	pub updates_len: usize,
}
