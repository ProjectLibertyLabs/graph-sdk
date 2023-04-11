#![allow(dead_code)] // todo: remove after usage
use crate::dsnp::{dsnp_types::DsnpUserId, encryption::EncryptionBehavior};
use std::{cmp::Ordering, fmt::Debug, hash::Hash};

/// Raw page of Graph (or Key) data
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct PageData {
	/// Id of the page
	pub page_id: PageId,

	/// raw content of page data
	pub content: Vec<u8>,

	/// hash value of content
	pub content_hash: PageHash,
}

#[derive(Clone, PartialEq, Eq, Debug)]
pub struct KeyData {
	/// index of the key stored on chain
	pub index: u16,

	/// raw content of key data
	pub content: Vec<u8>,
}

/// KeyPair used for encryption and PRI calculations
pub struct KeyPair<E: EncryptionBehavior> {
	/// Public key
	pub public_key: E::EncryptionInput,

	/// Private key
	pub private_key: E::DecryptionInput,
}

/// PublicKey type associated in `EncryptionBehavior`
pub type PublicKey<E> = <E as EncryptionBehavior>::EncryptionInput;

/// Privacy Type of the graph
#[derive(Clone, Copy, PartialEq, Ord, Eq, PartialOrd, Debug, Hash)]
pub enum PrivacyType {
	/// publicly accessible graph
	Public,

	/// only accessible to owner of the graph and whoever it is shared with
	Private,
}

/// Different connection type in social graph
#[derive(Clone, Copy, PartialEq, Ord, Eq, PartialOrd, Debug, Hash)]
pub enum ConnectionType {
	/// Follow is a one-way connection type, which means it is only stored in follower side
	Follow(PrivacyType),

	/// Friendship is two-way connection type, which means it is stored in both sides and each
	/// side can revoke the connection for both sides
	Friendship(PrivacyType),
}

/// Graph page id
pub type PageId = u16;

/// Schema ID
pub type SchemaId = u16;

/// Page Hash type
pub type PageHash = u32;

/// A trait defining configurable settings for sdk
pub trait Config {
	fn schema_for_connection_type(&self, connection_type: ConnectionType) -> SchemaId;
}

/// Encapsulates all the keys and page data that needs to be retrieved from chain
pub struct Import<E: EncryptionBehavior> {
	/// graph owner dsnp user id
	pub dsnp_user_id: DsnpUserId,

	/// graph keys associated with this graph which is used for encryption and PRI generation
	pub keys: Vec<KeyPair<E>>,

	/// Page data containing the social graph retrieved from chain
	pub pages: Vec<PageData>,
}

/// A connection representation in graph sdk
pub struct Connection {
	/// dsnp user id of the user that this connection is associated with
	pub dsnp_user_id: DsnpUserId,

	/// connection type
	pub connection_type: ConnectionType,
}

/// Encapsulates a dsnp user and their associated graph public keys
/// It is primarily used for PRI calculations
#[derive(Debug, PartialEq)]
pub struct DsnpKeys {
	/// dsnp user id
	pub dsnp_user_id: DsnpUserId,

	/// content hash of itemized page
	pub keys_hash: PageHash,

	/// public keys for the dsnp user
	pub keys: Vec<KeyData>,
}

/// Different kind of actions that can be applied to the graph
pub enum Action<E: EncryptionBehavior> {
	/// an action that defines adding a connection in the social graph
	Connect {
		/// owner of the social graph
		owner_dsnp_user_id: DsnpUserId,

		/// connection details
		connection: Connection,

		/// public key associated with the user in the connection
		/// included only if PRId calculation is required
		connection_key: Option<PublicKey<E>>,
	},

	/// an action that defines removing an existing connection from social graph
	Disconnect {
		/// owner of the social graph
		owner_dsnp_user_id: DsnpUserId,

		/// connection details
		connection: Connection,
	},
}

/// Output of graph sdk that defines the different updates that needs to be applied to chain
pub enum Update {
	/// A `PersistPage` type is used to upsert a page on the chain with latest changes
	PersistPage {
		/// owner of the social graph
		owner_dsnp_user_id: DsnpUserId,

		/// schema id of this change, each graph has it's own schema so it depends on the graph type
		/// which is modified
		schema_id: SchemaId,

		/// page id associated with changed page
		page_id: PageId,

		/// previous hash value is used to avoid updating a stale state
		prev_hash: Vec<u8>,

		/// social graph page data
		payload: Vec<u8>,
	},

	/// A `DeletePage` type is used to remove a page from the chain
	DeletePage {
		/// owner of the social graph
		owner_dsnp_user_id: DsnpUserId,

		/// schema id of this change, each graph has it's own schema so it depends on the graph type
		/// which is modified
		schema_id: SchemaId,

		/// page id associated with changed page
		page_id: PageId,

		/// previous hash value is used to avoid updating a stale state
		prev_hash: Vec<u8>,
	},

	/// A `Delete` type is used to remove a page from the chain
	AddKey {
		/// owner of the social graph
		owner_dsnp_user_id: DsnpUserId,

		/// schema id of location for keys
		schema_id: SchemaId,

		/// previous hash value is used to avoid updating a stale state
		prev_hash: Vec<u8>,

		/// social graph page data
		payload: Vec<u8>,
	},
}

/// Encapsulates details required to do a key rotation
pub struct Rotation<E: EncryptionBehavior> {
	/// owner of the social graph
	owner_dsnp_user_id: DsnpUserId,

	/// previous key used for encryption and PRI calculations
	prev_key: KeyPair<E>,

	/// new key to use for encryption and PRI calculations
	new_key: KeyPair<E>,
}

impl PartialOrd for KeyData {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		Some(self.cmp(other))
	}
}

impl Ord for KeyData {
	fn cmp(&self, other: &Self) -> Ordering {
		self.index.cmp(&other.index)
	}
}
