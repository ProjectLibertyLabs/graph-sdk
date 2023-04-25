use crate::dsnp::{dsnp_configs::KeyPairType, dsnp_types::DsnpUserId};
use dryoc::keypair::PublicKey as StackPublicKey;
pub use dsnp_graph_config::{ConnectionType, PrivacyType};
use dsnp_graph_config::{GraphKeyType, SchemaId};
use std::{cmp::Ordering, fmt::Debug};

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

/// Key Pair wrapper
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct GraphKeyPair {
	/// key pair type
	pub key_type: GraphKeyType,

	/// public key raw
	pub public_key: Vec<u8>,

	/// secret key raw
	pub secret_key: Vec<u8>,
}

/// A resolved KeyPair used for encryption and PRI calculations
#[derive(Clone, Debug)]
pub struct ResolvedKeyPair {
	/// Key identifier
	pub key_id: u64,

	/// Public key
	pub key_pair: KeyPairType,
}

/// Graph page id
pub type PageId = u16;

/// Page Hash type
pub type PageHash = u32;

/// Encapsulates all the decryption keys and page data that need to be retrieved from chain
pub struct ImportBundle {
	/// graph owner dsnp user id
	pub dsnp_user_id: DsnpUserId,

	/// Schema id of imported data
	pub schema_id: SchemaId,

	/// key pairs associated with this graph which is used for encryption and PRI generation
	pub key_pairs: Vec<GraphKeyPair>,

	/// published dsnp keys associated with this dsnp user
	pub dsnp_keys: DsnpKeys,

	/// Page data containing the social graph retrieved from chain
	pub pages: Vec<PageData>,
}

/// A connection representation in graph sdk
pub struct Connection {
	/// dsnp user id of the user that this connection is associated with
	pub dsnp_user_id: DsnpUserId,

	/// Schema id of imported data
	pub schema_id: SchemaId,
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
pub enum Action {
	/// an action that defines adding a connection in the social graph
	Connect {
		/// owner of the social graph
		owner_dsnp_user_id: DsnpUserId,

		/// connection details
		connection: Connection,

		/// public key associated with the user in the connection
		/// included only if PRId calculation is required
		connection_key: Option<StackPublicKey>,
	},

	/// an action that defines removing an existing connection from social graph
	Disconnect {
		/// owner of the social graph
		owner_dsnp_user_id: DsnpUserId,

		/// connection details
		connection: Connection,
	},
}

impl Action {
	pub fn owner_dsnp_user_id(&self) -> DsnpUserId {
		match *self {
			Action::Connect { owner_dsnp_user_id, .. } => owner_dsnp_user_id,
			Action::Disconnect { owner_dsnp_user_id, .. } => owner_dsnp_user_id,
		}
	}
}

/// Output of graph sdk that defines the different updates that needs to be applied to chain
#[allow(dead_code)]
pub enum Update {
	/// A `PersistPage` type is used to upsert a page on the chain with latest changes
	PersistPage {
		/// owner of the social graph
		owner_dsnp_user_id: DsnpUserId,

		/// Schema id of imported data
		schema_id: SchemaId,

		/// page id associated with changed page
		page_id: PageId,

		/// previous hash value is used to avoid updating a stale state
		prev_hash: PageHash,

		/// social graph page data
		payload: Vec<u8>,
	},

	/// A `DeletePage` type is used to remove a page from the chain
	DeletePage {
		/// owner of the social graph
		owner_dsnp_user_id: DsnpUserId,

		/// Schema id of removed data
		schema_id: SchemaId,

		/// page id associated with changed page
		page_id: PageId,

		/// previous hash value is used to avoid updating a stale state
		prev_hash: PageHash,
	},

	/// A `AddKey` type is used to add a new key to chain
	AddKey {
		/// owner of the social graph
		owner_dsnp_user_id: DsnpUserId,

		/// previous hash value is used to avoid updating a stale state
		prev_hash: PageHash,

		/// social graph page data
		payload: Vec<u8>,
	},
}

#[allow(dead_code)] // todo: use or remove
/// Encapsulates details required to do a key rotation
pub struct Rotation {
	/// owner of the social graph
	owner_dsnp_user_id: DsnpUserId,

	/// previous key used for encryption and PRI calculations
	prev_key: GraphKeyPair,

	/// new key to use for encryption and PRI calculations
	new_key: GraphKeyPair,
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

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn key_data_should_be_ordered_by_index_asc() {
		let a = KeyData { index: 1, content: vec![] };
		let b = KeyData { index: 19, content: vec![] };
		let c = KeyData { index: 20, content: vec![] };
		let mut arr = vec![b.clone(), a.clone(), c.clone()];

		arr.sort();

		assert_eq!(arr, vec![a, b, c]);
	}
}
