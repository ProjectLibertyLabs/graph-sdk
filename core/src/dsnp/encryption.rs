use crate::dsnp::dsnp_configs::{PublicKeyType, SecretKeyType};
use anyhow::{Error, Result as AnyResult};
use dryoc::{
	classic::crypto_box::{crypto_box_seal, crypto_box_seal_open},
	constants::CRYPTO_BOX_SEALBYTES,
	dryocbox::ByteArray,
};

/// Common trait for different encryption algorithms
pub trait EncryptionBehavior {
	/// encrypt the plain_data
	fn encrypt(&self, plain_data: &[u8], input: &PublicKeyType) -> AnyResult<Vec<u8>>;

	/// decrypt the encrypted_data
	fn decrypt(&self, encrypted_data: &[u8], input: &SecretKeyType) -> AnyResult<Vec<u8>>;
}

/// XSalsa20Poly1305 encryption algorithm
#[derive(Clone, Debug, Eq, PartialEq, Default, Hash)]
pub struct SealBox;

impl EncryptionBehavior for SealBox {
	fn encrypt(&self, plain_data: &[u8], input: &PublicKeyType) -> AnyResult<Vec<u8>> {
		match input {
			PublicKeyType::Version1_0(key) => {
				let mut encrypted =
					vec![0u8; plain_data.len().saturating_add(CRYPTO_BOX_SEALBYTES)];
				crypto_box_seal(&mut encrypted, plain_data, key.as_array())
					.map_err(|e| Error::msg(format!("failed to encrypt {:?}", e)))?;
				Ok(encrypted)
			},
		}
	}

	fn decrypt(&self, encrypted_data: &[u8], input: &SecretKeyType) -> AnyResult<Vec<u8>> {
		match input {
			SecretKeyType::Version1_0(key) => {
				let mut plain =
					vec![0u8; encrypted_data.len().saturating_sub(CRYPTO_BOX_SEALBYTES)];
				crypto_box_seal_open(
					plain.as_mut_slice(),
					encrypted_data,
					key.public_key.as_array(),
					key.secret_key.as_array(),
				)
				.map_err(|e| Error::msg(format!("failed to decrypt {:?}", e)))?;
				Ok(plain)
			},
		}
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::dsnp::dsnp_configs::KeyPairType;
	use dryoc::keypair::StackKeyPair;

	#[test]
	fn sealbox_should_encrypt_and_decrypt_successfully() {
		let plain_data = vec![
			23, 23, 109, 198, 111, 70, 2, 89, 2u8, 1, 0, 23, 5, 82, 100, 56, 1, 120, 200, 250, 140,
			83, 98, 0, 10, 234, 88, 23, 54, 23, 23, 109, 198, 111, 70, 2, 89,
		];

		let key_pair = KeyPairType::Version1_0(StackKeyPair::gen());
		let encrypted = SealBox.encrypt(&plain_data, &key_pair.clone().into()).unwrap();
		let decrypted = SealBox.decrypt(&encrypted, &key_pair.into()).unwrap();

		assert_eq!(decrypted, plain_data);
	}

	#[test]
	fn sealbox_decrypting_corrupted_data_should_fail() {
		let plain_data = vec![83, 98, 0, 10, 234, 88, 23, 54, 23, 23, 109, 198, 111, 70, 2, 89];

		let key_pair = KeyPairType::Version1_0(StackKeyPair::from_seed(&[0, 1, 2, 3, 4]));
		let mut encrypted = SealBox.encrypt(&plain_data, &key_pair.clone().into()).unwrap();
		encrypted[1] = encrypted[1].saturating_add(1); // corrupting data
		let decrypted = SealBox.decrypt(&encrypted, &key_pair.into());

		assert!(decrypted.is_err());
	}
}
