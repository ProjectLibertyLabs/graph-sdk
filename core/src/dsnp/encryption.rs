use anyhow::{Error, Result as AnyResult};
use dryoc::{
	classic::crypto_box::{crypto_box_seal, crypto_box_seal_open},
	constants::CRYPTO_BOX_SEALBYTES,
	dryocbox::{ByteArray, PublicKey},
	keypair::StackKeyPair,
};

/// Common trait for different encryption algorithms
pub trait EncryptionBehavior {
	/// encryption input type such as encryption key
	type EncryptionInput;
	/// decryption input type such as decryption key
	type DecryptionInput;

	/// encrypt the plain_data
	fn encrypt(plain_data: &[u8], input: &Self::EncryptionInput) -> AnyResult<Vec<u8>>;

	/// decrypt the encrypted_data
	fn decrypt(encrypted_data: &[u8], input: &Self::DecryptionInput) -> AnyResult<Vec<u8>>;
}

/// XSalsa20Poly1305 encryption algorithm
#[derive(Clone, Debug, Eq, PartialEq, Default)]
pub struct SealBox;

impl EncryptionBehavior for SealBox {
	type EncryptionInput = PublicKey;
	type DecryptionInput = StackKeyPair;

	fn encrypt(plain_data: &[u8], input: &Self::EncryptionInput) -> AnyResult<Vec<u8>> {
		let mut encrypted = vec![0u8; plain_data.len().saturating_add(CRYPTO_BOX_SEALBYTES)];
		crypto_box_seal(&mut encrypted, plain_data, input.as_array())
			.map_err(|e| Error::msg(format!("failed to encrypt {:?}", e)))?;
		Ok(encrypted)
	}

	fn decrypt(encrypted_data: &[u8], input: &Self::DecryptionInput) -> AnyResult<Vec<u8>> {
		let mut plain = vec![0u8; encrypted_data.len().saturating_sub(CRYPTO_BOX_SEALBYTES)];
		crypto_box_seal_open(
			plain.as_mut_slice(),
			encrypted_data,
			input.public_key.as_array(),
			input.secret_key.as_array(),
		)
		.map_err(|e| Error::msg(format!("failed to decrypt {:?}", e)))?;
		Ok(plain)
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use dryoc::keypair::StackKeyPair;

	#[test]
	fn sealbox_should_encrypt_and_decrypt_successfully() {
		let plain_data = vec![
			23, 23, 109, 198, 111, 70, 2, 89, 2u8, 1, 0, 23, 5, 82, 100, 56, 1, 120, 200, 250, 140,
			83, 98, 0, 10, 234, 88, 23, 54, 23, 23, 109, 198, 111, 70, 2, 89,
		];

		let key_pair = StackKeyPair::gen();
		let encrypted = SealBox::encrypt(&plain_data, &key_pair.public_key).unwrap();
		let decrypted = SealBox::decrypt(&encrypted, &key_pair).unwrap();

		assert_eq!(decrypted, plain_data);
	}

	#[test]
	fn sealbox_decrypting_corrupted_data_should_fail() {
		let plain_data = vec![83, 98, 0, 10, 234, 88, 23, 54, 23, 23, 109, 198, 111, 70, 2, 89];

		let key_pair = StackKeyPair::from_seed(&[0, 1, 2, 3, 4]);
		let mut encrypted = SealBox::encrypt(&plain_data, &key_pair.public_key).unwrap();
		encrypted[1] = encrypted[1].saturating_add(1); // corrupting data
		let decrypted = SealBox::decrypt(&encrypted, &key_pair);

		assert!(decrypted.is_err());
	}
}
