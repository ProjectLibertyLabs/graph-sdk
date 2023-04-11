use crate::{
	dsnp::{
		api_types::{DsnpKeys, KeyData, PageHash},
		dsnp_types::{DsnpPublicKey, DsnpUserId},
		reader_writer::{DsnpReader, DsnpWriter},
	},
	frequency::Frequency,
};
use anyhow::{Error, Result};
use dryoc::keypair::StackKeyPair;
use std::{cell::RefCell, collections::HashMap, ops::Deref, rc::Rc, sync::RwLock};

trait PublicKeyProvider {
	fn import_dsnp_keys(&mut self, keys: DsnpKeys) -> Result<()>;
	fn export_new_keys(&self) -> Result<Vec<DsnpKeys>>;
	fn get_all_keys(&self, dsnp_user_id: DsnpUserId) -> Vec<&DsnpPublicKey>;
	fn get_key_by_id(&self, dsnp_user_id: DsnpUserId, key_id: u64) -> Option<&DsnpPublicKey>;
	fn get_key_by_public_key(
		&self,
		dsnp_user_id: DsnpUserId,
		public_key: Vec<u8>,
	) -> Option<&DsnpPublicKey>;
	fn get_active_key(&self, dsnp_user_id: DsnpUserId) -> Option<&DsnpPublicKey>;
	fn add_new_key(&mut self, dsnp_user_id: DsnpUserId, public_key: Vec<u8>) -> Result<u64>;
}

trait UserKeyProvider {
	fn import_key_pairs(&mut self, pairs: Vec<StackKeyPair>);
	fn get_key(&self, key_id: u64) -> Option<(DsnpPublicKey, StackKeyPair)>;
}

pub struct PublicKeyManager {
	/// keys are stored sorted by index
	dsnp_user_to_keys: HashMap<DsnpUserId, (Vec<DsnpPublicKey>, PageHash)>,
	new_keys: HashMap<DsnpUserId, DsnpPublicKey>,
}

pub struct UserKeyManager {
	public_key_manager: RwLock<PublicKeyManager>,
	dsnp_user_id: DsnpUserId,
	keys: Vec<StackKeyPair>,
}

impl PublicKeyProvider for PublicKeyManager {
	/// importing dsnp keys as they are retrieved from blockchain
	/// sorting indices since id's might not be unique but indices definitely should be
	fn import_dsnp_keys(&mut self, keys: DsnpKeys) -> Result<()> {
		let mut sorted_keys = keys.keys.clone().to_vec();
		// sorting by index in ascending mode
		sorted_keys.sort();

		let mut dsnp_keys = vec![];
		for key in sorted_keys {
			let k = Frequency::read_public_key(&key.content)
				.map_err(|e| Error::msg(format!("failed to deserialize key {:?}", e)))?;
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
					// this is not important since it's not being used for writing on chain
					index: u16::default(),
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
		// What should we do if there are more than one key with the same key_id uploaded?
		// currently choosing the first one
		self.get_all_keys(dsnp_user_id).into_iter().find(|k| k.key_id == key_id)
	}

	fn get_key_by_public_key(
		&self,
		dsnp_user_id: DsnpUserId,
		public_key: Vec<u8>,
	) -> Option<&DsnpPublicKey> {
		self.get_all_keys(dsnp_user_id).into_iter().find(|k| k.key == public_key)
	}

	fn get_active_key(&self, dsnp_user_id: DsnpUserId) -> Option<&DsnpPublicKey> {
		self.get_all_keys(dsnp_user_id).last().copied()
	}

	fn add_new_key(&mut self, dsnp_user_id: DsnpUserId, public_key: Vec<u8>) -> Result<u64> {
		let new_key = DsnpPublicKey { key_id: self.get_next_key_id(dsnp_user_id), key: public_key };

		// making sure it is serializable before adding
		let _ = Frequency::write_public_key(&new_key)
			.map_err(|e| Error::msg(format!("failed to serialize key {:?}", e)))?;

		// only one new key is allowed to be added to a dsnp_user_id at a time
		self.new_keys.insert(dsnp_user_id, new_key.clone());

		Ok(new_key.key_id)
	}
}

impl UserKeyProvider for UserKeyManager {
	fn import_key_pairs(&mut self, pairs: Vec<StackKeyPair>) {
		self.keys.clear();
		self.keys.extend_from_slice(&pairs);
	}

	fn get_key(&self, key_id: u64) -> Option<(DsnpPublicKey, StackKeyPair)> {
		if let Some(dsnp) =
			self.public_key_manager.read().unwrap().get_key_by_id(self.dsnp_user_id, key_id)
		{
			if let Some(pair) = self.keys.iter().find(|&k| k.public_key.to_vec() == dsnp.key) {
				return Some((dsnp.clone(), pair.clone()))
			}
		}
		None
	}
}

impl PublicKeyManager {
	fn new() -> Self {
		Self { new_keys: HashMap::new(), dsnp_user_to_keys: HashMap::new() }
	}

	fn get_next_key_id(&self, dsnp_user_id: DsnpUserId) -> u64 {
		self.get_all_keys(dsnp_user_id)
			.iter()
			.map(|key| key.key_id)
			.max()
			.unwrap_or(u64::default()) +
			1
	}
}

impl UserKeyManager {
	pub fn new(public_key_manager: RwLock<PublicKeyManager>, dsnp_user_id: DsnpUserId) -> Self {
		Self { public_key_manager, dsnp_user_id, keys: vec![] }
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::borrow::Borrow;

	fn create_dsnp_keys(
		dsnp_user_id: DsnpUserId,
		keys_hash: PageHash,
		key_data: Vec<KeyData>,
	) -> DsnpKeys {
		DsnpKeys { keys_hash, dsnp_user_id, keys: key_data }
	}

	#[test]
	fn public_key_manager_should_import_and_retrieve_keys_as_expected() {
		// arrange
		let dsnp_user_id = 23;
		let key1 = DsnpPublicKey { key_id: 128, key: b"217678127812871812334324".to_vec() };
		let serialized1 = Frequency::write_public_key(&key1).expect("should serialize");
		let key2 = DsnpPublicKey { key_id: 1, key: b"217678127812871812334325".to_vec() };
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
		assert_eq!(key_manager.get_key_by_id(dsnp_user_id, 128), Some(&key1));
		assert_eq!(key_manager.get_key_by_public_key(dsnp_user_id, key1.key.clone()), Some(&key1));
		assert_eq!(key_manager.get_key_by_public_key(dsnp_user_id, key2.key.clone()), Some(&key2));
		assert_eq!(key_manager.get_active_key(dsnp_user_id), Some(&key1));
	}

	#[test]
	fn public_key_manager_add_new_key_should_store_a_key_with_increased_id() {
		// arrange
		let dsnp_user_id = 2;
		let keys_hash = 233;
		let key1 = DsnpPublicKey { key_id: 19, key: b"217678127812871812334324".to_vec() };
		let serialized1 = Frequency::write_public_key(&key1).expect("should serialize");
		let key2 = DsnpPublicKey { key_id: 4, key: b"217678127812871812334325".to_vec() };
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
		let expected_added_key = DsnpPublicKey { key_id: 20, key: new_public_key.clone() };
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
					index: 0,
					content: Frequency::write_public_key(&expected_added_key)
						.expect("should write")
				}]
			}]
		);
		assert_eq!(key_manager.get_all_keys(dsnp_user_id).len(), 3);
	}

	// #[test]
	// fn user_key_manager_should_import_and_retrieve_keys_as_expected() {
	// 	// arrange
	// 	let dsnp_user_id = 2;
	// 	let mut public_key_manager = PublicKeyManager::new();
	// 	let mut rc = RwLock::new(&public_key_manager);
	// 	let mut user_key_manager = UserKeyManager::new(rc.read().unwrap().clone(), dsnp_user_id);
	// 	let key_pair = StackKeyPair::gen();
	// 	let keys_hash = 233;
	// 	let key1 = DsnpPublicKey { key_id: 19, key: key_pair.public_key.to_vec() };
	// 	let serialized1 = Frequency::write_public_key(&key1).expect("should serialize");
	// 	let keys = create_dsnp_keys(
	// 		dsnp_user_id,
	// 		keys_hash,
	// 		vec![
	// 			KeyData { index: 1, content: serialized1 },
	// 		],
	// 	);
	// 	rc.get_mut().unwrap().import_dsnp_keys(keys).expect("should work");
	//
	// 	// act
	// 	user_key_manager.import_key_pairs(vec![key_pair.clone()]);
	//
	// 	// assert
	// 	let key = user_key_manager.get_key(key1.key_id);
	// 	assert_eq!(key, Some((&key1, &key_pair)));
	// }
}
