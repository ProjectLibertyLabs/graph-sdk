use crate::dsnp::dsnp_types::DsnpPrid;
use crypto_box::{aead::AeadInPlace, Nonce, PublicKey, SecretKey};
use hkdf::Hkdf;
use sha2::Sha256;
use std::ops::Deref;
use x25519_dalek::x25519;
use xsalsa20poly1305::{KeyInit, XSalsa20Poly1305};
use zeroize::{Zeroize, Zeroizing};

const PRI_CONTEXT: &[u8] = b"PRIdCtx0";
struct PseudonymousRelationshipIdentifier;

impl PseudonymousRelationshipIdentifier {
	fn create(a: u64, b: u64, a_secret_key: &SecretKey, b_public_key: &PublicKey) -> DsnpPrid {
		let id_b = b.to_le_bytes();

		let mut root_shared =
			Zeroizing::new(x25519(*a_secret_key.as_bytes(), *b_public_key.as_bytes()));
		let hk = Hkdf::<Sha256>::new(Some(&id_b[..]), root_shared.as_slice());
		let mut derived_key = Zeroizing::new([0u8; 32]);
		hk.expand(PRI_CONTEXT, derived_key.as_mut_slice()).unwrap();
		root_shared.zeroize();

		let salsa = XSalsa20Poly1305::new(derived_key.deref().into());

		let mut nonce = [0u8; 24];
		nonce.copy_from_slice(&[&[0u8; 16], &id_b[..]].concat());
		let nonce = Nonce::from(nonce);

		let mut buffer = a.to_le_bytes();
		salsa.encrypt_in_place_detached(&nonce, &[], &mut buffer).unwrap();
		derived_key.zeroize();

		DsnpPrid::new(&buffer)
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
		);
		let pri_a_to_b_2 = PseudonymousRelationshipIdentifier::create(
			a,
			b,
			&secret_key_b,
			&secret_key_a.public_key(),
		);

		println!("{:?}  {:?}", pri_a_to_b, pri_a_to_b_2);
		assert_eq!(pri_a_to_b, pri_a_to_b_2);
	}
}
