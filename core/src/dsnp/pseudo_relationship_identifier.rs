#![allow(dead_code)] // todo: remove after usage
use crate::dsnp::dsnp_types::{DsnpId, DsnpPrid};
use anyhow::{Error, Result};
use crypto_box::{aead::AeadInPlace, Nonce, PublicKey, SecretKey};
use hkdf::Hkdf;
use sha2::Sha256;
use std::ops::Deref;
use x25519_dalek::StaticSecret;
use xsalsa20poly1305::{KeyInit, XSalsa20Poly1305};
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
		let id_b = b.to_le_bytes();

		// calculate shared secret
		let root_shared = StaticSecret::from(*a_secret_key.as_bytes())
			.diffie_hellman(&x25519_dalek::PublicKey::from(*b_public_key.as_bytes()));

		// derive a new key form pri context
		let mut derived_key = Zeroizing::new([0u8; 32]);
		let hk = Hkdf::<Sha256>::new(Some(&id_b[..]), root_shared.as_bytes());
		hk.expand(PRI_CONTEXT, derived_key.as_mut_slice())
			.map_err(|e| Error::msg(format!("key derivation error {:?}", e)))?;

		// setting nonce with `b` for encryption
		let mut nonce = [0u8; 24];
		nonce[..8].copy_from_slice(&id_b[..]);
		let nonce = Nonce::from(nonce);

		// encrypting `a` using nonce and derived key
		let mut result = a.to_le_bytes();
		let salsa = XSalsa20Poly1305::new(derived_key.deref().into());
		salsa
			.encrypt_in_place_detached(&nonce, &[], &mut result)
			.map_err(|e| Error::msg(format!("failed to encrypt detached {:?}", e)))?;

		Ok(DsnpPrid::new(&result))
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crypto_box::{aead::OsRng, SecretKey};

	#[test]
	fn generated_pri_should_be_the_same_calculated_from_both_sides() {
		let a = 2576367222u64;
		let b = 826378782u64;
		let secret_key_a = SecretKey::generate(&mut OsRng);
		let secret_key_b = SecretKey::generate(&mut OsRng);

		let pri_a_to_b = PseudonymousRelationshipIdentifier::create(
			a,
			b,
			&secret_key_a,
			&secret_key_b.public_key(),
		)
		.expect("should create pri");
		let pri_a_to_b_2 = PseudonymousRelationshipIdentifier::create(
			a,
			b,
			&secret_key_b,
			&secret_key_a.public_key(),
		)
		.expect("should create pri");

		println!("{:?}  {:?}", pri_a_to_b, pri_a_to_b_2);
		assert_eq!(pri_a_to_b, pri_a_to_b_2);
	}
}
