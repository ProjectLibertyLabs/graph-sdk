use dsnp_graph_config::GraphKeyType;
use libc::{c_void, size_t};

/// KeyData wrapper
#[repr(C)]
pub struct KeyData {
	index: u16,
	content: *mut u8,
}

/// Key Pair wrapper
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

// Raw page of Graph (or Key) data
#[repr(C)]
pub struct PageData {
	// Id of the page
	pub page_id: u16,

	// raw content of page data
	pub content: *mut u8,
	pub content_len: size_t,

	// hash value of content
	pub content_hash: u64,
}

// GraphAPI wrapper
#[repr(C)]
pub struct DsnpKeys {
	dsnp_user_id: u64,
	keys_hash: u64,
	keys: *mut KeyData,
	keys_len: size_t,
}

// ImportBundle wrapper
#[repr(C)]
pub struct ImportBundle {
	/// graph owner dsnp user id
	pub dsnp_user_id: u64,

	/// Schema id of imported data
	pub schema_id: u16,

	/// key pairs associated with this graph which is used for encryption and PRI generation
	pub key_pairs: *mut GraphKeyPair,
	pub key_pairs_len: size_t,

	/// published dsnp keys associated with this dsnp user
	pub dsnp_keys: *mut DsnpKeys,

	/// Page data containing the social graph retrieved from chain
	pub pages: *mut PageData,
	pub pages_len: size_t,
}

// Connection wrapper
#[repr(C)]
pub struct Connection {
	// dsnp user id
	pub dsnp_user_id: u64,
	// schema id
	pub schema_id: u16,
}

// ActionType wrapper
#[repr(C)]
pub enum ActionType {
	// Connect action
	Connect,
	// Disconnect action
	Disconnect,
}

// ConnectAction wrapper
#[repr(C)]
pub struct ConnectAction {
	// dsnp user id
	owner_dsnp_user_id: u64,
	// connection
	connection: Connection,
}

// DisconnectAction wrapper
#[repr(C)]
pub struct DisconnectAction {
	// dsnp user id
	owner_dsnp_user_id: u64,
	// connection
	connection: Connection,
}

// Action wrapper
#[repr(C)]
pub struct Action {
	// action type
	action_type: ActionType,
	// connect action
	action: *mut c_void,
}

// DsnpGraphEdge wrapper
#[repr(C)]
pub struct DsnpGraphEdge {
	// dsnp user id
	user_id: u64,
	// connection since
	since: u64,
}

#[repr(C)]
pub struct PersistPage {
	/// owner of the social graph
	owner_dsnp_user_id: u64,

	/// Schema id of imported data
	schema_id: u16,

	/// page id associated with changed page
	page_id: u16,

	/// previous hash value is used to avoid updating a stale state
	prev_hash: u32,

	/// social graph page data
	payload: *mut u8,
	payload_len: size_t,
}

#[repr(C)]
pub struct DeletePage {
	/// owner of the social graph
	owner_dsnp_user_id: u64,

	/// Schema id of removed data
	schema_id: u16,

	/// page id associated with changed page
	page_id: u16,

	/// previous hash value is used to avoid updating a stale state
	prev_hash: u32,
}

#[repr(C)]
pub struct AddKey {
	/// owner of the social graph
	owner_dsnp_user_id: u64,

	/// previous hash value is used to avoid updating a stale state
	prev_hash: u32,

	/// social graph page data
	payload: *mut u8,
	payload_len: size_t,
}

#[repr(C)]
pub enum Update {
	PersistPage(PersistPage),
	DeletePage(DeletePage),
	AddKey(AddKey),
}

#[repr(C)]
pub enum ConnectionType {
	Follow(PrivacyType),
	Friendship(PrivacyType),
}

#[repr(C)]
pub enum PrivacyType {
	Public,
	Private,
}

#[repr(C)]
pub enum DsnpVersion {
	Version1_0,
}

#[repr(C)]
pub struct SchemaConfig {
	pub dsnp_version: DsnpVersion,
	pub connection_type: ConnectionType,
}

#[repr(C)]
pub struct Config {
	pub sdk_max_users_graph_size: u32,
	pub max_graph_page_size_bytes: u32,
	pub max_page_id: u32,
	pub max_key_page_size_bytes: u32,
	pub schema_map_len: size_t,
	pub schema_map: *mut (u16, SchemaConfig),
	pub dsnp_versions_len: size_t,
	pub dsnp_versions: *mut DsnpVersion,
}
