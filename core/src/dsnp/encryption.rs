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
#[derive(Clone, Debug, Eq, PartialEq)]
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
