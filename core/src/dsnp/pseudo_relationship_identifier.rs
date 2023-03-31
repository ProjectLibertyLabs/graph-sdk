#![allow(dead_code)] // todo: remove after usage
use crate::dsnp::dsnp_types::{DsnpId, DsnpPrid};
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
struct PseudonymousRelationshipIdentifier;

impl PseudonymousRelationshipIdentifier {
	fn create(
		a: DsnpId,
		b: DsnpId,
		a_secret_key: &SecretKey,
		b_public_key: &PublicKey,
	) -> Result<DsnpPrid> {
		let id_a = a.to_le_bytes();
		let id_b = b.to_le_bytes();

		// calculate shared secret
		let root_shared =
			Zeroizing::new(crypto_box_beforenm(b_public_key.as_array(), a_secret_key.as_array()));

		// // derive a new key form pri context
		let kdf =
			StackKdf::from_parts(Key::from(root_shared.deref()), PRI_CONTEXT.as_array().into());
		let derived_key: Zeroizing<Key> = Zeroizing::new(
			kdf.derive_subkey(b)
				.map_err(|e| Error::msg(format!("key derivation error {:?}", e)))?,
		);

		// setting nonce with `b` for encryption
		let mut nonce = Nonce::default();
		nonce[..8].copy_from_slice(&id_b[..]);

		// encrypting `a` using nonce and derived key
		let mut result = vec![0u8; id_a.len()];
		let mut _mac = [0u8; CRYPTO_SECRETBOX_MACBYTES];
		crypto_secretbox_detached(
			&mut result,
			&mut _mac,
			&id_a,
			&nonce,
			derived_key.deref().as_array(),
		);

		Ok(DsnpPrid::new(&result))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use dryoc::keypair::StackKeyPair;

	#[test]
	fn generated_pri_should_be_the_same_calculated_from_both_sides() {
		let a = 2576367222u64;
		let b = 826378782u64;
		let key_pair_a = StackKeyPair::gen();
		let key_pair_b = StackKeyPair::gen();

		let pri_a_to_b = PseudonymousRelationshipIdentifier::create(
			a,
			b,
			&key_pair_a.secret_key,
			&key_pair_b.public_key,
		)
		.expect("should create pri");
		let pri_a_to_b_2 = PseudonymousRelationshipIdentifier::create(
			a,
			b,
			&key_pair_b.secret_key,
			&key_pair_a.public_key,
		)
		.expect("should create pri");

		println!("{:?}  {:?}", pri_a_to_b, pri_a_to_b_2);
		assert_eq!(pri_a_to_b, pri_a_to_b_2);
	}
}
