#![allow(dead_code)] // todo: remove after usage
use crate::dsnp::dsnp_types::{DsnpPrid, DsnpUserId};
use anyhow::{Error, Result};
use dryoc::{
	classic::{
		crypto_box::crypto_box_beforenm,
		crypto_secretbox::{crypto_secretbox_detached, Nonce},
	},
	constants::CRYPTO_SECRETBOX_MACBYTES,
	kdf::{Key, StackKdf},
	keypair::{PublicKey, SecretKey},
	types::ByteArray,
};
use std::ops::Deref;
use zeroize::Zeroizing;

const PRI_CONTEXT: &[u8] = b"PRIdCtx0";

/// a trait that implements the PRI related algorithm
pub trait PridProvider {
	/// Return type of Prid
	type DsnpPrid;

	/// creates PRId from A -> B
	fn create_prid(
		a: DsnpUserId,
		b: DsnpUserId,
		a_secret_key: &SecretKey,
		b_public_key: &PublicKey,
	) -> Result<Self::DsnpPrid>;

	/// creates shared context from A -> B
	fn create_shared_context(
		b: DsnpUserId,
		a_secret_key: &SecretKey,
		b_public_key: &PublicKey,
	) -> Result<SecretKey>;
}

impl PridProvider for DsnpPrid {
	type DsnpPrid = DsnpPrid;

	fn create_prid(
		a: DsnpUserId,
		b: DsnpUserId,
		a_secret_key: &SecretKey,
		b_public_key: &PublicKey,
	) -> Result<Self::DsnpPrid> {
		let id_a = a.to_le_bytes();
		let id_b = b.to_le_bytes();
		let shared_context = Self::create_shared_context(b, a_secret_key, b_public_key)?;

		// setting nonce with `a` for encryption
		let mut nonce = Nonce::default();
		nonce[..8].copy_from_slice(&id_a[..]);

		// encrypting `b` using nonce and derived key
		let mut result = vec![0u8; id_b.len()];
		let mut _mac = [0u8; CRYPTO_SECRETBOX_MACBYTES];
		crypto_secretbox_detached(
			&mut result,
			&mut _mac,
			&id_b,
			&nonce,
			shared_context.deref().as_array(),
		);

		Ok(Self::DsnpPrid::new(&result))
	}

	fn create_shared_context(
		b: DsnpUserId,
		a_secret_key: &SecretKey,
		b_public_key: &PublicKey,
	) -> Result<Key> {
		// calculate shared secret
		let root_shared =
			Zeroizing::new(crypto_box_beforenm(b_public_key.as_array(), a_secret_key.as_array()));

		// // derive a new key form pri context
		let kdf =
			StackKdf::from_parts(Key::from(root_shared.deref()), PRI_CONTEXT.as_array().into());
		let derived_key: Key = kdf
			.derive_subkey(b)
			.map_err(|e| Error::msg(format!("key derivation error {:?}", e)))?;

		Ok(derived_key)
	}
}
