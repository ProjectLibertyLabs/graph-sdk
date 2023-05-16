use crate::dsnp::{dsnp_configs::KeyPairType, dsnp_types::DsnpUserId};
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
#[derive(Clone, Debug, PartialEq)]
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
#[derive(Debug, Clone)]
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

/// Encapsulates a dsnp user and their associated graph public keys
/// It is primarily used for PRI calculations
#[repr(C)]
#[derive(Debug, PartialEq, Clone)]
pub struct DsnpKeys {
	/// dsnp user id
	pub dsnp_user_id: DsnpUserId,

	/// content hash of itemized page
	pub keys_hash: PageHash,

	/// public keys for the dsnp user
	pub keys: Vec<KeyData>,
}

/// A connection representation in graph sdk
#[repr(C)]
#[derive(Debug, Clone)]
pub struct Connection {
	/// dsnp user id of the user that this connection is associated with
	pub dsnp_user_id: DsnpUserId,

	/// Schema id of imported data
	pub schema_id: SchemaId,
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

		/// optional keys to import for the connection. Mostly useful for private friendships.
		dsnp_keys: Option<DsnpKeys>,
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
		new_public_key: Vec<u8>,
	},
}

impl Action {
	pub fn owner_dsnp_user_id(&self) -> DsnpUserId {
		match *self {
			Action::Connect { owner_dsnp_user_id, .. } => owner_dsnp_user_id,
			Action::Disconnect { owner_dsnp_user_id, .. } => owner_dsnp_user_id,
			Action::AddGraphKey { owner_dsnp_user_id, .. } => owner_dsnp_user_id,
		}
	}
}

/// Output of graph sdk that defines the different updates that needs to be applied to chain
#[derive(Debug, Clone, PartialEq)]
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

/// converts a `PageData` type to `Update` type
impl From<(PageData, DsnpUserId, SchemaId)> for Update {
	fn from((page_data, owner_dsnp_user_id, schema_id): (PageData, DsnpUserId, SchemaId)) -> Self {
		match page_data.content.is_empty() {
			false => Update::PersistPage {
				owner_dsnp_user_id,
				schema_id,
				page_id: page_data.page_id,
				prev_hash: page_data.content_hash,
				payload: page_data.content.clone(),
			},
			true => Update::DeletePage {
				owner_dsnp_user_id,
				schema_id,
				page_id: page_data.page_id,
				prev_hash: page_data.content_hash,
			},
		}
	}
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

	#[test]
	fn update_from_page_data_should_create_correct_update_types() {
		// arrange
		let dsnp_user_id = 3;
		let schema_id = 10;
		let persist =
			PageData { page_id: 1, content: vec![1u8, 2, 3, 4, 5, 6, 7, 8, 9], content_hash: 182 };
		let delete = PageData {
			page_id: 2,
			content: vec![], // empty content should generate `DeletePage`
			content_hash: 345,
		};

		// act
		let persist_update = Update::from((persist, dsnp_user_id, schema_id));
		let delete_update = Update::from((delete, dsnp_user_id, schema_id));

		// assert
		assert!(matches!(persist_update, Update::PersistPage { .. }));
		assert!(matches!(delete_update, Update::DeletePage { .. }));
	}
}
