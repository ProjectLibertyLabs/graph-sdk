use crate::{
	dsnp::{
		api_types::{DsnpKeys, KeyData, PageHash, ResolvedKeyPair},
		dsnp_configs::KeyPairType,
		dsnp_types::{DsnpPublicKey, DsnpUserId},
		reader_writer::{DsnpReader, DsnpWriter},
	},
	frequency::Frequency,
};
use anyhow::{Error, Result};
use std::{cell::RefCell, collections::HashMap, rc::Rc};

/// A trait that defines all the functionality that a public key provider need to implement.
pub trait PublicKeyProvider {
	/// imports public keys with their hash and details into the provider
	/// will overwrite any existing imported keys for the user and remove any new added keys
	fn import_dsnp_keys(&mut self, keys: DsnpKeys) -> Result<()>;

	/// adds a new public key to the provider
	fn add_new_key(&mut self, dsnp_user_id: DsnpUserId, public_key: Vec<u8>) -> Result<()>;

	/// exports added new keys to be submitted to chain
	fn export_new_keys(&self) -> Result<Vec<DsnpKeys>>;

	/// get imported and new keys. New keys are appended in the end.
	fn get_all_keys(&self, dsnp_user_id: DsnpUserId) -> Vec<&DsnpPublicKey>;

	/// returns a key by its id
	fn get_key_by_id(&self, dsnp_user_id: DsnpUserId, key_id: u64) -> Option<&DsnpPublicKey>;

	/// returns a key by its public key
	fn get_key_by_public_key(
		&self,
		dsnp_user_id: DsnpUserId,
		public_key: Vec<u8>,
	) -> Option<&DsnpPublicKey>;

	/// returns the active key for a a user to used for encryption
	fn get_active_key(&self, dsnp_user_id: DsnpUserId) -> Option<&DsnpPublicKey>;
}

/// Common trait that manages public and private keys for each user
pub trait UserKeyProvider {
	/// imports key pairs into a provider
	/// will overwrite any existing imported keys for the user
	fn import_key_pairs(&mut self, pairs: Vec<KeyPairType>);

	/// returns the dsnp associate and keypair with a certain id
	fn get_resolved_key(&self, key_id: u64) -> Option<(DsnpPublicKey, KeyPairType)>;

	/// returns the dsnp associate and keypair with all the keys
	fn get_all_resolved_keys(&self) -> Vec<ResolvedKeyPair>;

	/// returns the active key for a a user to used for encryption
	fn get_resolved_active_key(
		&self,
		dsnp_user_id: DsnpUserId,
	) -> Option<(DsnpPublicKey, KeyPairType)>;
}

#[derive(Debug, Eq, PartialEq)]
pub struct PublicKeyManager {
	/// keys are stored sorted by index
	dsnp_user_to_keys: HashMap<DsnpUserId, (Vec<DsnpPublicKey>, PageHash)>,

	/// stores and keeps track of any new key being added
	new_keys: HashMap<DsnpUserId, DsnpPublicKey>,
}

#[derive(Debug)]
pub struct UserKeyManager {
	/// keeps a reference to the global instance of public key provider
	public_key_manager: Rc<RefCell<PublicKeyManager>>,

	/// current user dsnp id that this key manager belongs to
	dsnp_user_id: DsnpUserId,

	/// key pairs associated with this user
	keys: Vec<KeyPairType>,
}

impl PublicKeyProvider for PublicKeyManager {
	/// importing dsnp keys as they are retrieved from blockchain
	/// sorting indices since ids might not be unique but indices definitely should be
	fn import_dsnp_keys(&mut self, keys: DsnpKeys) -> Result<()> {
		self.dsnp_user_to_keys.remove(&keys.dsnp_user_id);
		self.new_keys.remove(&keys.dsnp_user_id);

		let mut sorted_keys = keys.keys.clone().to_vec();
		// sorting by index in ascending mode
		sorted_keys.sort();

		let mut dsnp_keys = vec![];
		for key in sorted_keys {
			let mut k = Frequency::read_public_key(&key.content)
				.map_err(|e| Error::msg(format!("failed to deserialize key {:?}", e)))?;
			// key id is the itemized index of the key stored in Frequency
			k.key_id = Some(key.index.into());
			dsnp_keys.push(k);
		}

		self.dsnp_user_to_keys.insert(keys.dsnp_user_id, (dsnp_keys, keys.keys_hash));
		Ok(())
	}

	fn export_new_keys(&self) -> Result<Vec<DsnpKeys>> {
		let mut result = vec![];
		for (dsnp_user_id, key) in &self.new_keys {
			result.push(DsnpKeys {
				dsnp_user_id: *dsnp_user_id,
				keys: vec![KeyData {
					content: Frequency::write_public_key(&key)
						.map_err(|e| Error::msg(format!("failed to serialize key {:?}", e)))?,
					// all new keys are assigned a new id so we can unwrap here
					index: key.key_id.unwrap() as u16,
				}],
				keys_hash: self
					.dsnp_user_to_keys
					.get(&dsnp_user_id)
					.expect("Key hash should exist")
					.1,
			});
		}
		Ok(result)
	}

	fn get_all_keys(&self, dsnp_user_id: DsnpUserId) -> Vec<&DsnpPublicKey> {
		let mut all_keys = vec![];
		if let Some((v, _)) = self.dsnp_user_to_keys.get(&dsnp_user_id) {
			all_keys.extend(&v[..]);
		}
		if let Some(k) = self.new_keys.get(&dsnp_user_id) {
			all_keys.push(k)
		}
		all_keys
	}

	fn get_key_by_id(&self, dsnp_user_id: DsnpUserId, key_id: u64) -> Option<&DsnpPublicKey> {
		// get the first key by that id as specified in the spec
		self.get_all_keys(dsnp_user_id)
			.iter()
			.find(|k| k.key_id == Some(key_id))
			.copied()
	}

	fn get_key_by_public_key(
		&self,
		dsnp_user_id: DsnpUserId,
		public_key: Vec<u8>,
	) -> Option<&DsnpPublicKey> {
		// get the first key by that public key as specified in the spec
		self.get_all_keys(dsnp_user_id).iter().find(|k| k.key == public_key).copied()
	}

	fn get_active_key(&self, dsnp_user_id: DsnpUserId) -> Option<&DsnpPublicKey> {
		let last_key = self.get_all_keys(dsnp_user_id).last().cloned();
		if let Some(k) = last_key {
			if let Some(key_id) = k.key_id {
				// get the first key published by that key_id
				return self.get_key_by_id(dsnp_user_id, key_id)
			}
		}
		last_key
	}

	fn add_new_key(&mut self, dsnp_user_id: DsnpUserId, public_key: Vec<u8>) -> Result<()> {
		let new_key =
			DsnpPublicKey { key: public_key, key_id: Some(self.get_next_key_id(dsnp_user_id)) };

		// making sure it is serializable before adding
		let _ = Frequency::write_public_key(&new_key)
			.map_err(|e| Error::msg(format!("failed to serialize key {:?}", e)))?;

		// only one new key is allowed to be added to a dsnp_user_id at a time
		self.new_keys.insert(dsnp_user_id, new_key.clone());

		Ok(())
	}
}

impl UserKeyProvider for UserKeyManager {
	fn import_key_pairs(&mut self, pairs: Vec<KeyPairType>) {
		self.keys.clear();
		self.keys.extend_from_slice(&pairs);
	}

	fn get_resolved_key(&self, key_id: u64) -> Option<(DsnpPublicKey, KeyPairType)> {
		if let Some(dsnp) =
			self.public_key_manager.borrow().get_key_by_id(self.dsnp_user_id, key_id)
		{
			if let Some(pair) = self.keys.iter().find(|&k| k.get_public_key_raw() == dsnp.key) {
				return Some((dsnp.clone(), pair.clone()))
			}
		}
		None
	}

	fn get_all_resolved_keys(&self) -> Vec<ResolvedKeyPair> {
		self.public_key_manager
			.borrow()
			.get_all_keys(self.dsnp_user_id)
			.iter()
			.filter_map(|dsnp| match dsnp.key_id {
				Some(ind) => self.get_resolved_key(ind),
				None => None,
			})
			.map(|(dsnp, pair)| ResolvedKeyPair {
				key_id: dsnp.key_id.unwrap(),
				key_pair: pair.clone(),
			})
			.collect()
	}

	fn get_resolved_active_key(
		&self,
		dsnp_user_id: DsnpUserId,
	) -> Option<(DsnpPublicKey, KeyPairType)> {
		if let Some(key) = self.public_key_manager.borrow().get_active_key(dsnp_user_id) {
			// can unwrap here since public key returns all keys with their ids
			if let Some(res) = self.get_resolved_key(key.key_id.unwrap()) {
				return Some(res)
			}
		}
		None
	}
}

impl PublicKeyManager {
	pub fn new() -> Self {
		Self { new_keys: HashMap::new(), dsnp_user_to_keys: HashMap::new() }
	}

	fn get_next_key_id(&self, dsnp_user_id: DsnpUserId) -> u64 {
		self.get_all_keys(dsnp_user_id)
			.iter()
			.filter_map(|key| key.key_id)
			.max()
			.unwrap_or(u64::default()) +
			1
	}
}

impl UserKeyManager {
	pub fn new(
		dsnp_user_id: DsnpUserId,
		public_key_manager: Rc<RefCell<PublicKeyManager>>,
	) -> Self {
		Self { public_key_manager, dsnp_user_id, keys: vec![] }
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use dryoc::keypair::StackKeyPair;

	fn create_dsnp_keys(
		dsnp_user_id: DsnpUserId,
		keys_hash: PageHash,
		key_data: Vec<KeyData>,
	) -> DsnpKeys {
		DsnpKeys { keys_hash, dsnp_user_id, keys: key_data }
	}

	#[test]
	fn public_key_manager_import_should_clean_previous_keys() {
		// arrange
		let mut key_manager = PublicKeyManager::new();
		let dsnp_user_id = 23;
		let key_hash = 128;
		let key1 = DsnpPublicKey { key_id: Some(128), key: b"217678127812871812334324".to_vec() };
		let serialized1 = Frequency::write_public_key(&key1).expect("should serialize");
		let old_keys = create_dsnp_keys(
			dsnp_user_id,
			key_hash,
			vec![KeyData { index: 2, content: serialized1 }],
		);
		key_manager.import_dsnp_keys(old_keys).expect("should work");
		key_manager
			.add_new_key(dsnp_user_id, b"21767812782988871812334324".to_vec())
			.expect("should add new key");

		// act
		let _ = key_manager.import_dsnp_keys(create_dsnp_keys(dsnp_user_id, key_hash, vec![]));

		// assert
		assert_eq!(key_manager.dsnp_user_to_keys.get(&dsnp_user_id), Some(&(Vec::new(), key_hash)));
		assert_eq!(key_manager.new_keys.get(&dsnp_user_id), None);
	}

	#[test]
	fn public_key_manager_should_import_and_retrieve_keys_as_expected() {
		// arrange
		let dsnp_user_id = 23;
		let key1 = DsnpPublicKey { key_id: Some(2), key: b"217678127812871812334324".to_vec() };
		let serialized1 = Frequency::write_public_key(&key1).expect("should serialize");
		let key2 = DsnpPublicKey { key_id: Some(1), key: b"217678127812871812334325".to_vec() };
		let serialized2 = Frequency::write_public_key(&key2).expect("should serialize");
		let keys = create_dsnp_keys(
			dsnp_user_id,
			17826,
			vec![
				KeyData { index: 2, content: serialized1 },
				KeyData { index: 1, content: serialized2 },
			],
		);
		let mut key_manager = PublicKeyManager::new();

		// act
		let res = key_manager.import_dsnp_keys(keys);

		// assert
		assert!(res.is_ok());
		assert_eq!(key_manager.get_key_by_id(dsnp_user_id, 1), Some(&key2));
		assert_eq!(key_manager.get_key_by_id(dsnp_user_id, 2), Some(&key1));
		assert_eq!(key_manager.get_key_by_public_key(dsnp_user_id, key1.key.clone()), Some(&key1));
		assert_eq!(key_manager.get_key_by_public_key(dsnp_user_id, key2.key.clone()), Some(&key2));
		assert_eq!(key_manager.get_active_key(dsnp_user_id), Some(&key1));
	}

	#[test]
	fn public_key_manager_add_new_key_should_store_a_key_with_increased_id() {
		// arrange
		let dsnp_user_id = 2;
		let keys_hash = 233;
		let key1 = DsnpPublicKey { key_id: None, key: b"217678127812871812334324".to_vec() };
		let serialized1 = Frequency::write_public_key(&key1).expect("should serialize");
		let key2 = DsnpPublicKey { key_id: None, key: b"217678127812871812334325".to_vec() };
		let serialized2 = Frequency::write_public_key(&key2).expect("should serialize");
		let keys = create_dsnp_keys(
			dsnp_user_id,
			keys_hash,
			vec![
				KeyData { index: 1, content: serialized1 },
				KeyData { index: 2, content: serialized2 },
			],
		);
		let new_public_key = b"726871hsjgdjsa727821712812".to_vec();
		let expected_added_key = DsnpPublicKey { key_id: Some(3), key: new_public_key.clone() };
		let mut key_manager = PublicKeyManager::new();
		key_manager.import_dsnp_keys(keys).expect("should work");

		// act
		let res = key_manager.add_new_key(dsnp_user_id, new_public_key.clone());

		// assert
		assert!(res.is_ok());
		let active_key = key_manager.get_active_key(dsnp_user_id);
		assert_eq!(active_key, Some(&expected_added_key));
		let export = key_manager.export_new_keys().expect("should work");
		assert_eq!(
			export,
			vec![DsnpKeys {
				keys_hash,
				dsnp_user_id,
				keys: vec![KeyData {
					index: expected_added_key.key_id.unwrap() as u16,
					content: Frequency::write_public_key(&expected_added_key)
						.expect("should write")
				}]
			}]
		);
		assert_eq!(key_manager.get_all_keys(dsnp_user_id).len(), 3);
	}

	#[test]
	fn public_key_manager_get_key_by_id_should_return_first_key_when_duplicate_ids_exists() {
		// arrange
		let dsnp_user_id = 2;
		let id = 4;
		let key1 = DsnpPublicKey { key_id: Some(id), key: b"217678127812871812334324".to_vec() };
		let serialized1 = Frequency::write_public_key(&key1).expect("should serialize");
		let key2 = DsnpPublicKey { key_id: None, key: b"217678127812871812334325".to_vec() };
		let serialized2 = Frequency::write_public_key(&key2).expect("should serialize");
		let keys = create_dsnp_keys(
			dsnp_user_id,
			233,
			vec![
				KeyData { index: id as u16, content: serialized1 },
				KeyData { index: id as u16, content: serialized2 },
			],
		);
		let mut key_manager = PublicKeyManager::new();
		key_manager.import_dsnp_keys(keys).expect("should work");

		// act
		let res = key_manager.get_key_by_id(dsnp_user_id, id.into());

		// assert
		assert_eq!(res, Some(&key1));
	}

	#[test]
	fn user_key_manager_should_import_and_retrieve_keys_as_expected() {
		// arrange
		let dsnp_user_id = 2;
		let public_key_manager = PublicKeyManager::new();
		let rc = Rc::new(RefCell::new(public_key_manager));
		let mutable_clone = rc.clone();
		let mut user_key_manager = UserKeyManager::new(dsnp_user_id, rc.clone());
		let key_pair = KeyPairType::Version1_0(StackKeyPair::gen());
		let keys_hash = 233;
		let id1 = 1;
		let key1 = DsnpPublicKey { key_id: Some(id1), key: key_pair.get_public_key_raw() };
		let serialized1 = Frequency::write_public_key(&key1).expect("should serialize");
		let keys = create_dsnp_keys(
			dsnp_user_id,
			keys_hash,
			vec![KeyData { index: id1 as u16, content: serialized1 }],
		);
		mutable_clone.borrow_mut().import_dsnp_keys(keys).expect("should work");

		// act
		user_key_manager.import_key_pairs(vec![key_pair.clone()]);

		// assert
		let key = user_key_manager.get_resolved_key(id1);
		assert_eq!(key, Some((key1.clone(), key_pair.clone())));

		let keys = user_key_manager.get_all_resolved_keys();
		assert_eq!(keys.len(), 1);

		let resolved_active = user_key_manager.get_resolved_active_key(dsnp_user_id);
		assert_eq!(resolved_active, Some((key1, key_pair)));
	}
}
