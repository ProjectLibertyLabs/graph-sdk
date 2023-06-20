//! Different structs and types used in API
use crate::dsnp::{dsnp_configs::KeyPairType, dsnp_types::DsnpUserId};
use dsnp_graph_config::{
	errors::{
		DsnpGraphError::{
			InvalidDsnpUserId, InvalidInput, InvalidPublicKey, InvalidSchemaId, InvalidSecretKey,
		},
		DsnpGraphResult,
	},
	GraphKeyType, InputValidation, SchemaId,
};
pub use dsnp_graph_config::{ConnectionType, PageId, PrivacyType};
use log::Level;
use log_result_proc_macro::log_result_err;
use serde::{Deserialize, Serialize};
use std::{cmp::Ordering, collections::HashSet, fmt::Debug};

/// Page Hash type
pub type PageHash = u32;

/// Raw page of Graph (or Key) data
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct PageData {
	/// Id of the page
	#[serde(rename = "pageId")]
	pub page_id: PageId,

	/// raw content of page data
	#[serde(rename = "content")]
	pub content: Vec<u8>,

	/// hash value of content
	#[serde(rename = "contentHash")]
	pub content_hash: PageHash,
}

/// implementing input validation for Page Data
impl InputValidation for PageData {
	#[log_result_err(Level::Info)]
	fn validate(&self) -> DsnpGraphResult<()> {
		if self.content.len() > 0 && self.content_hash == PageHash::default() {
			return DsnpGraphResult::Err(InvalidInput(format!(
				"Imported Page Data and page hash {0} does not match!",
				self.content_hash
			)))
		}

		Ok(())
	}
}

/// Represents a published graph key
#[derive(Clone, PartialEq, Eq, Debug, Serialize, Deserialize)]
pub struct KeyData {
	/// index of the key stored on chain
	#[serde(rename = "index")]
	pub index: u16,

	/// raw content of key data
	#[serde(rename = "content")]
	pub content: Vec<u8>,
}

/// implementing input validation for key Data
impl InputValidation for KeyData {
	#[log_result_err(Level::Info)]
	fn validate(&self) -> DsnpGraphResult<()> {
		if self.content.is_empty() {
			return DsnpGraphResult::Err(InvalidInput("key_data content is empty!".to_string()))
		}
		Ok(())
	}
}

/// Key-pair wrapper provided by wallet
#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct GraphKeyPair {
	/// key pair type
	#[serde(rename = "keyType")]
	pub key_type: GraphKeyType,

	/// public key raw
	#[serde(rename = "publicKey")]
	pub public_key: Vec<u8>,

	/// secret key raw
	#[serde(rename = "secretKey")]
	pub secret_key: Vec<u8>,
}

/// implementing input validation for import bundle
impl InputValidation for GraphKeyPair {
	#[log_result_err(Level::Info)]
	fn validate(&self) -> DsnpGraphResult<()> {
		if self.public_key.is_empty() {
			return DsnpGraphResult::Err(InvalidPublicKey)
		}
		if self.secret_key.is_empty() {
			return DsnpGraphResult::Err(InvalidSecretKey)
		}
		Ok(())
	}
}

/// A resolved KeyPair used for encryption and PRI calculations
#[derive(Clone, Debug, PartialEq)]
pub struct ResolvedKeyPair {
	/// Key identifier
	pub key_id: u64,

	/// Public key
	pub key_pair: KeyPairType,
}

/// Encapsulates all the decryption keys and page data that need to be retrieved from chain
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportBundle {
	/// graph owner dsnp user id
	#[serde(rename = "dsnpUserId")]
	pub dsnp_user_id: DsnpUserId,

	/// Schema id of imported data
	#[serde(rename = "schemaId")]
	pub schema_id: SchemaId,

	/// key pairs associated with this graph which is used for encryption and PRI generation
	#[serde(rename = "keyPairs")]
	pub key_pairs: Vec<GraphKeyPair>,

	/// published dsnp keys associated with this dsnp user
	#[serde(rename = "dsnpKeys")]
	pub dsnp_keys: DsnpKeys,

	/// Page data containing the social graph retrieved from chain
	#[serde(rename = "pages")]
	pub pages: Vec<PageData>,
}

/// implementing input validation for import bundle
impl InputValidation for ImportBundle {
	#[log_result_err(Level::Info)]
	fn validate(&self) -> DsnpGraphResult<()> {
		if self.dsnp_user_id == 0 {
			return DsnpGraphResult::Err(InvalidDsnpUserId(self.dsnp_user_id))
		}
		if self.schema_id == 0 {
			return DsnpGraphResult::Err(InvalidSchemaId(self.schema_id))
		}

		for k in &self.key_pairs {
			k.validate()?;
		}

		self.dsnp_keys.validate()?;

		for p in &self.pages {
			p.validate()?;
		}

		let unique_pages: HashSet<_> = self.pages.iter().map(|p| p.page_id).collect();
		if unique_pages.len() < self.pages.len() {
			return DsnpGraphResult::Err(InvalidInput("Duplicated pageId in PageData".to_string()))
		}

		Ok(())
	}
}

/// Encapsulates a dsnp user and their associated graph public keys
/// It is primarily used for PRI calculations
#[repr(C)]
#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct DsnpKeys {
	/// dsnp user id
	#[serde(rename = "dsnpUserId")]
	pub dsnp_user_id: DsnpUserId,

	/// content hash of itemized page
	#[serde(rename = "keysHash")]
	pub keys_hash: PageHash,

	/// public keys for the dsnp user
	#[serde(rename = "keys")]
	pub keys: Vec<KeyData>,
}

/// implementing input validation for Dsnp Keys
impl InputValidation for DsnpKeys {
	#[log_result_err(Level::Info)]
	fn validate(&self) -> DsnpGraphResult<()> {
		if self.dsnp_user_id == 0 {
			return DsnpGraphResult::Err(InvalidDsnpUserId(self.dsnp_user_id))
		}

		if self.keys.len() > 0 && self.keys_hash == PageHash::default() {
			return DsnpGraphResult::Err(InvalidInput(format!(
				"Imported Keys and page hash {0} does not match!",
				self.keys_hash
			)))
		}

		for k in &self.keys {
			k.validate()?;
		}
		let unique_indices: HashSet<_> = self.keys.iter().map(|k| k.index).collect();
		if unique_indices.len() < self.keys.len() {
			return DsnpGraphResult::Err(InvalidInput("Duplicated key index in KeyData".to_string()))
		}

		Ok(())
	}
}

/// A connection representation in graph sdk
#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Connection {
	/// dsnp user id of the user that this connection is associated with
	#[serde(rename = "dsnpUserId")]
	pub dsnp_user_id: DsnpUserId,

	/// Schema id of imported data
	#[serde(rename = "schemaId")]
	pub schema_id: SchemaId,
}

/// implementing input validation for Connection
impl InputValidation for Connection {
	#[log_result_err(Level::Info)]
	fn validate(&self) -> DsnpGraphResult<()> {
		if self.dsnp_user_id == 0 {
			return DsnpGraphResult::Err(InvalidDsnpUserId(self.dsnp_user_id))
		}
		if self.schema_id == 0 {
			return DsnpGraphResult::Err(InvalidSchemaId(self.schema_id))
		}

		Ok(())
	}
}

/// Different kind of actions that can be applied to the graph
#[repr(C)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Action {
	/// an action that defines adding a connection in the social graph
	Connect {
		/// owner of the social graph
		#[serde(rename = "ownerDsnpUserId")]
		owner_dsnp_user_id: DsnpUserId,

		/// connection details
		#[serde(rename = "connection")]
		connection: Connection,

		/// optional keys to import for the connection. Mostly useful for private friendships.
		#[serde(rename = "dsnpKeys")]
		dsnp_keys: Option<DsnpKeys>,
	},

	/// an action that defines removing an existing connection from social graph
	Disconnect {
		/// owner of the social graph
		#[serde(rename = "ownerDsnpUserId")]
		owner_dsnp_user_id: DsnpUserId,

		/// connection details
		#[serde(rename = "connection")]
		connection: Connection,
	},

	/// an action that defines adding a new key to chain
	AddGraphKey {
		/// owner of the social graph
		#[serde(rename = "ownerDsnpUserId")]
		owner_dsnp_user_id: DsnpUserId,

		/// public key
		#[serde(rename = "newPublicKey")]
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

/// implementing input validation for Action
impl InputValidation for Action {
	#[log_result_err(Level::Info)]
	fn validate(&self) -> DsnpGraphResult<()> {
		if self.owner_dsnp_user_id() == 0 {
			return DsnpGraphResult::Err(InvalidDsnpUserId(self.owner_dsnp_user_id()))
		}

		match self {
			Action::Connect { connection, dsnp_keys, .. } => {
				connection.validate()?;

				if let Some(keys) = dsnp_keys {
					keys.validate()?;
				}
			},
			Action::Disconnect { connection, .. } => {
				connection.validate()?;
			},
			Action::AddGraphKey { new_public_key, .. } =>
				if new_public_key.is_empty() {
					return DsnpGraphResult::Err(InvalidPublicKey)
				},
		}

		Ok(())
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
