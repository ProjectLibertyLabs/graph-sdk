use anyhow::{Error, Result as AnyResult};
use crypto_box::aead::OsRng;

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
pub struct SealBox;

impl EncryptionBehavior for SealBox {
	type EncryptionInput = crypto_box::PublicKey;
	type DecryptionInput = crypto_box::SecretKey;

	fn encrypt(plain_data: &[u8], input: &Self::EncryptionInput) -> AnyResult<Vec<u8>> {
		let encrypted = crypto_box::seal(&mut OsRng, input, plain_data)
			.map_err(|e| Error::msg(format!("failed to encrypt {:?}", e)))?;
		Ok(encrypted)
	}

	fn decrypt(encrypted_data: &[u8], input: &Self::DecryptionInput) -> AnyResult<Vec<u8>> {
		let encrypted = crypto_box::seal_open(input, encrypted_data)
			.map_err(|e| Error::msg(format!("failed to decrypt {:?}", e)))?;
		Ok(encrypted)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crypto_box::SecretKey;

	#[test]
	fn sealbox_should_encrypt_and_decrypt_successfully() {
		let plain_data = vec![
			23, 23, 109, 198, 111, 70, 2, 89, 2u8, 1, 0, 23, 5, 82, 100, 56, 1, 120, 200, 250, 140,
			83, 98, 0, 10, 234, 88, 23, 54, 23, 23, 109, 198, 111, 70, 2, 89,
		];

		let secret_key = SecretKey::generate(&mut OsRng);
		let encrypted = SealBox::encrypt(&plain_data, &secret_key.public_key()).unwrap();
		let decrypted = SealBox::decrypt(&encrypted, &secret_key).unwrap();

		assert_eq!(decrypted, plain_data);
	}

	#[test]
	fn sealbox_decrypting_corrupted_data_should_fail() {
		let plain_data = vec![83, 98, 0, 10, 234, 88, 23, 54, 23, 23, 109, 198, 111, 70, 2, 89];

		let secret_key = SecretKey::generate(&mut OsRng);
		let mut encrypted = SealBox::encrypt(&plain_data, &secret_key.public_key()).unwrap();
		encrypted[1] = 9; // corrupting data
		let decrypted = SealBox::decrypt(&encrypted, &secret_key);

		assert!(decrypted.is_err());
	}
}
