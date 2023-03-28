#![allow(dead_code)] // todo: remove after usage
use crate::dsnp::{dsnp_types::DsnpId, encryption::EncryptionBehavior};

/// Raw page of Graph (or Key) data
pub type Page = Vec<u8>;

pub struct KeyPair<E: EncryptionBehavior> {
	/// Public key
	pub public_key: E::EncryptionInput,

	/// Private key
	pub private_key: E::DecryptionInput,
}

pub type PublicKey<E> = <E as EncryptionBehavior>::EncryptionInput;

pub enum PrivacyType {
	Public,
	Private,
}

pub enum ConnectionType {
	Follow(PrivacyType),
	Friendship(PrivacyType),
}

pub trait Config {
	/// Graph page id
	type PageId;
	/// Schema ID
	type SchemaId;

	const PUBLIC_FOLLOW_SCHEMAID: Self::SchemaId;
	const PRIVATE_FOLLOW_SCHEMAID: Self::SchemaId;
	const PUBLIC_FRIEND_SCHEMAID: Self::SchemaId;
	const PRIVATE_FRIEND_SCHEMAID: Self::SchemaId;

	fn schema_for_connection_type(connection_type: ConnectionType) -> Self::SchemaId {
		match connection_type {
			ConnectionType::Follow(PrivacyType::Public) => Self::PUBLIC_FOLLOW_SCHEMAID,
			ConnectionType::Follow(PrivacyType::Private) => Self::PRIVATE_FOLLOW_SCHEMAID,
			ConnectionType::Friendship(PrivacyType::Public) => Self::PUBLIC_FRIEND_SCHEMAID,
			ConnectionType::Friendship(PrivacyType::Private) => Self::PRIVATE_FRIEND_SCHEMAID,
		}
	}
}

pub struct Import<E: EncryptionBehavior> {
	pub dsnp_user_id: DsnpId,
	pub keys: Vec<KeyPair<E>>,
	pub pages: Vec<Page>,
}

pub struct Connection {
	pub dsnp_user_id: DsnpId,
	pub connection_type: ConnectionType,
}

pub struct DsnpKeys<E: EncryptionBehavior> {
	pub dsnp_user_id: DsnpId,
	pub keys: Vec<PublicKey<E>>,
}

pub enum Action<E: EncryptionBehavior> {
	Connect {
		owner_dsnp_user_id: DsnpId,
		connection: Connection,
		connection_key: Option<PublicKey<E>>, // included only if PRId calculation is required
	},
	Disconnect {
		owner_dsnp_user_id: DsnpId,
		connection: Connection,
	},
}

pub enum Update<C: Config> {
	Persist {
		owner_dsnp_user_id: DsnpId,
		schema_id: C::SchemaId,
		page_id: C::PageId,
		prev_hash: Vec<u8>,
		payload: Vec<u8>,
	},
	Delete {
		owner_dsnp_user_id: DsnpId,
		schema_id: C::SchemaId,
		page_id: C::PageId,
		prev_hash: Vec<u8>,
	},
}

pub struct Rotation<E: EncryptionBehavior> {
	owner_dsnp_user_id: DsnpId,
	prev_key: KeyPair<E>,
	new_key: KeyPair<E>,
}
