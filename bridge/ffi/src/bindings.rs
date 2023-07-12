use dsnp_graph_config::{DsnpVersion, GraphKeyType, SchemaConfig, SchemaId};
use dsnp_graph_core::{
	api::api_types::{Connection, PageHash, PageId},
	dsnp::dsnp_types::{DsnpGraphEdge, DsnpUserId},
};
use libc::size_t;

/// `dsnp_graph_core::dsnp::api_types::KeyData` type
#[repr(C)]
pub struct KeyData {
	pub index: u16,
	pub content: *mut u8,
	pub content_len: size_t,
}

/// `dsnp_graph_core::dsnp::dsnp_types::DsnpPublicKey` type
#[repr(C)]
pub struct DsnpPublicKey {
	pub key_id: u64,
	pub content: *mut u8,
	pub content_len: size_t,
}

/// Output type for `dsnp_graph_core::dsnp::api_types::DsnpPublicKey` list
#[repr(C)]
pub struct DsnpPublicKeys {
	pub keys: *mut DsnpPublicKey,
	pub keys_len: usize,
}

/// `dsnp_graph_core::dsnp::api_types::GraphKeyPair` type
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

/// `dsnp_graph_core::dsnp::api_types::PageData` type
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

/// `dsnp_graph_core::dsnp::api_types::DsnpKeys` type
#[repr(C)]
pub struct DsnpKeys {
	pub dsnp_user_id: DsnpUserId,
	pub keys_hash: PageHash,
	pub keys: *mut KeyData,
	pub keys_len: size_t,
}

/// `dsnp_graph_core::dsnp::api_types::ImportBundle` type
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
	pub dsnp_keys: Option<DsnpKeys>,

	/// Page data containing the social graph retrieved from chain
	pub pages: *mut PageData,
	pub pages_len: size_t,
}

/// `dsnp_graph_core::dsnp::api_types::Update::PersistPage` type
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

/// `dsnp_graph_core::dsnp::api_types::Update::DeletePage` type
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

// `dsnp_graph_core::dsnp::api_types::Update::AddKey` type
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

//// `dsnp_graph_core::dsnp::api_types::Update` type
#[repr(C)]
pub enum Update {
	Persist(PersistPage),
	Delete(DeletePage),
	Add(AddKey),
}

/// `dsnp_graph_core::dsnp::api_types::SchemaConfig` type
#[repr(C)]
pub struct SchemaConfigTuple {
	pub schema_id: SchemaId,
	pub schema_config: SchemaConfig,
}

/// `dsnp_graph_config::Config` type
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Config {
	pub sdk_max_stale_friendship_days: u32,
	pub max_graph_page_size_bytes: u32,
	pub max_page_id: u32,
	pub max_key_page_size_bytes: u32,
	pub schema_map_len: size_t,
	pub schema_map: *mut SchemaConfigTuple,
	pub graph_public_key_schema_id: SchemaId,
	pub dsnp_versions_len: size_t,
	pub dsnp_versions: *mut DsnpVersion,
}

/// `Environment` type for `Config`
#[repr(C)]
pub enum Environment {
	Mainnet,
	Rococo,
	Dev(Config),
}

/// Output type for`dsnp_graph_core::dsnp::dsn_types::DsnpGraphEdge` list
#[repr(C)]
pub struct GraphConnections {
	pub connections: *mut DsnpGraphEdge,
	pub connections_len: usize,
}

/// Output type for `dsnp_graph_core::dsnp::dsn_types::DsnpUserId` list
#[repr(C)]
pub struct GraphConnectionsWithoutKeys {
	pub connections: *mut DsnpUserId,
	pub connections_len: usize,
}

/// Output type for `dsnp_graph_core::dsnp::api_types::Update`
#[repr(C)]
pub struct GraphUpdates {
	pub updates: *mut Update,
	pub updates_len: usize,
}

/// Different kind of actions that can be applied to the graph
#[repr(C)]
#[derive(Debug, Clone)]
pub enum Action {
	/// an action that defines adding a connection in the social graph
	Connect {
		/// owner of the social graph
		owner_dsnp_user_id: DsnpUserId,

		/// connection details
		connection: Connection,

		/// optional key to import
		dsnp_keys: *mut DsnpKeys,
	},

	/// an action that defines removing an existing connection from social graph
	Disconnect {
		/// owner of the social graph
		owner_dsnp_user_id: DsnpUserId,

		/// connection details
		connection: Connection,
	},

	/// an action that defines adding a new key to chain
	AddGraphKey {
		/// owner of the social graph
		owner_dsnp_user_id: DsnpUserId,

		/// public key
		new_public_key: *const u8,
		new_public_key_len: size_t,
	},
}
