use crate::{
	api::api_types::{GraphKeyPair, PageData, ResolvedKeyPair},
	dsnp::{
		dsnp_configs::{KeyPairType, SecretKeyType},
		dsnp_types::{DsnpPrid, DsnpUserId},
		pseudo_relationship_identifier::PridProvider,
	},
	graph::shared_state_manager::{PriProvider, PublicKeyProvider, SharedStateManager},
	util::{transactional_hashmap::Transactional, transactional_vec::TransactionalVec},
};
use dsnp_graph_config::errors::DsnpGraphResult;
use std::{
	fmt::Debug,
	sync::{Arc, RwLock},
};

/// Common trait that manages public and private keys for each user
pub trait UserKeyProvider {
	/// imports key pairs into a provider
	/// will overwrite any existing imported keys for the user
	fn import_key_pairs(&mut self, pairs: Vec<GraphKeyPair>) -> DsnpGraphResult<()>;

	/// returns the dsnp associate and keypair with a certain id
	fn get_resolved_key(&self, key_id: u64) -> Option<ResolvedKeyPair>;

	/// returns the dsnp associate and keypair with all the keys
	fn get_all_resolved_keys(&self) -> Vec<ResolvedKeyPair>;

	/// returns the active key for a a user to used for encryption
	fn get_resolved_active_key(&self, dsnp_user_id: DsnpUserId) -> Option<ResolvedKeyPair>;
}

pub trait ConnectionVerifier {
	fn verify_connection(&self, from: DsnpUserId) -> DsnpGraphResult<bool>;
}

/// a combining trait that provides all functionalities required by user key manager
pub trait UserKeyManagerBase: UserKeyProvider + PriProvider + ConnectionVerifier + Debug {}

#[derive(Debug)]
pub struct UserKeyManager {
	/// keeps a reference to the shared instance of shared public keys and PRIDs
	shared_state_manager: Arc<RwLock<SharedStateManager>>,

	/// current user dsnp id that this key manager belongs to
	dsnp_user_id: DsnpUserId,

	/// key pairs associated with this user
	keys: TransactionalVec<KeyPairType>,
}

impl UserKeyProvider for UserKeyManager {
	fn import_key_pairs(&mut self, pairs: Vec<GraphKeyPair>) -> DsnpGraphResult<()> {
		let mut mapped_keys = vec![];
		for p in pairs {
			mapped_keys.push(p.try_into()?);
		}

		self.keys.clear();
		self.keys.extend_from_slice(&mapped_keys);

		Ok(())
	}

	fn get_resolved_key(&self, key_id: u64) -> Option<ResolvedKeyPair> {
		if let Some(dsnp) = self
			.shared_state_manager
			.read()
			.unwrap()
			.get_key_by_id(self.dsnp_user_id, key_id)
		{
			if let Some(key_pair) =
				self.keys.inner().iter().find(|&k| k.get_public_key_raw() == dsnp.key)
			{
				return Some(ResolvedKeyPair { key_id, key_pair: key_pair.clone() })
			}
		}
		None
	}

	fn get_all_resolved_keys(&self) -> Vec<ResolvedKeyPair> {
		self.shared_state_manager
			.read()
			.unwrap()
			.get_imported_keys(self.dsnp_user_id)
			.iter()
			.filter_map(|dsnp| match dsnp.key_id {
				Some(ind) => self.get_resolved_key(ind),
				None => None,
			})
			.collect()
	}

	fn get_resolved_active_key(&self, dsnp_user_id: DsnpUserId) -> Option<ResolvedKeyPair> {
		if let Some(key) = self.shared_state_manager.read().unwrap().get_active_key(dsnp_user_id) {
			// can unwrap here since public key returns all keys with their ids
			let key_id = key.key_id.unwrap();
			return self.get_resolved_key(key_id)
		}
		None
	}
}

impl PriProvider for UserKeyManager {
	fn import_pri(&mut self, dsnp_user_id: DsnpUserId, pages: &[PageData]) -> DsnpGraphResult<()> {
		self.shared_state_manager.write().unwrap().import_pri(dsnp_user_id, pages)
	}

	fn contains(&self, dsnp_user_id: DsnpUserId, prid: DsnpPrid) -> bool {
		self.shared_state_manager.read().unwrap().contains(dsnp_user_id, prid)
	}

	fn calculate_prid(
		&self,
		from: DsnpUserId,
		to: DsnpUserId,
		from_secret: SecretKeyType,
	) -> DsnpGraphResult<DsnpPrid> {
		self.shared_state_manager.read().unwrap().calculate_prid(from, to, from_secret)
	}
}

impl ConnectionVerifier for UserKeyManager {
	fn verify_connection(&self, from: DsnpUserId) -> DsnpGraphResult<bool> {
		let from_public_keys: Vec<_> = self
			.shared_state_manager
			.read()
			.unwrap()
			.get_prid_associated_public_keys(from)?;
		let to_resolved_keys = self.get_all_resolved_keys();

		for public in from_public_keys {
			for private in to_resolved_keys.iter().rev() {
				let prid = DsnpPrid::create_prid(
					from,
					self.dsnp_user_id,
					&private.key_pair.clone().into(),
					&public,
				)?;
				if self.shared_state_manager.read().unwrap().contains(from, prid) {
					return Ok(true)
				}
			}
		}
		Ok(false)
	}
}

impl UserKeyManagerBase for UserKeyManager {}

impl Transactional for UserKeyManager {
	fn commit(&mut self) {
		self.keys.commit();
	}

	fn rollback(&mut self) {
		self.keys.rollback();
	}
}

impl UserKeyManager {
	/// creates a new instance of `UserKeyManager`
	pub fn new(
		dsnp_user_id: DsnpUserId,
		public_key_manager: Arc<RwLock<SharedStateManager>>,
	) -> Self {
		Self {
			shared_state_manager: public_key_manager,
			dsnp_user_id,
			keys: TransactionalVec::new(),
		}
	}

	#[cfg(test)]
	pub fn get_imported_keys(&self) -> &Vec<KeyPairType> {
		self.keys.inner()
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::{
		api::api_types::{DsnpKeys, KeyData},
		dsnp::{dsnp_types::DsnpPublicKey, reader_writer::DsnpWriter},
		frequency::Frequency,
	};
	use dryoc::keypair::StackKeyPair;
	use dsnp_graph_config::GraphKeyType;

	#[test]
	fn user_key_manager_should_import_and_retrieve_keys_as_expected() {
		// arrange
		let dsnp_user_id = 2;
		let public_key_manager = SharedStateManager::new();
		let rc = Arc::new(RwLock::new(public_key_manager));
		let mutable_clone = rc.clone();
		let mut user_key_manager = UserKeyManager::new(dsnp_user_id, rc.clone());
		let key_pair_raw = StackKeyPair::gen();
		let key_pair_type = KeyPairType::Version1_0(key_pair_raw.clone());
		let key_pair = GraphKeyPair {
			secret_key: key_pair_raw.secret_key.to_vec(),
			public_key: key_pair_raw.public_key.to_vec(),
			key_type: GraphKeyType::X25519,
		};
		let keys_hash = 233;
		let id1 = 1;
		let key1 = DsnpPublicKey { key_id: Some(id1), key: key_pair.clone().public_key };
		let serialized1 = Frequency::write_public_key(&key1).expect("should serialize");
		let keys = DsnpKeys {
			keys_hash,
			dsnp_user_id,
			keys: vec![KeyData { index: id1 as u16, content: serialized1 }],
		};
		mutable_clone.write().unwrap().import_dsnp_keys(&keys).expect("should work");

		// act
		let res = user_key_manager.import_key_pairs(vec![key_pair.clone()]);

		// assert
		assert!(res.is_ok());
		let key = user_key_manager.get_resolved_key(id1);
		assert_eq!(key, Some(ResolvedKeyPair { key_id: id1, key_pair: key_pair_type.clone() }));

		let keys = user_key_manager.get_all_resolved_keys();
		assert_eq!(keys.len(), 1);

		let resolved_active = user_key_manager.get_resolved_active_key(dsnp_user_id);
		assert_eq!(resolved_active, Some(ResolvedKeyPair { key_id: id1, key_pair: key_pair_type }));
	}
}
