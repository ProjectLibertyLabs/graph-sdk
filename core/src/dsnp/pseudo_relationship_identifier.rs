//! Implementation of [PRIds](https://spec.dsnp.org/DSNP/UserData.html#private-connection-prids) defined in DSNP spec
use crate::dsnp::{
	dsnp_configs::{PublicKeyType, SecretKeyType},
	dsnp_types::{DsnpPrid, DsnpUserId},
};
use dryoc::{
	classic::{
		crypto_box::crypto_box_beforenm,
		crypto_secretbox::{crypto_secretbox_detached, Nonce},
	},
	constants::CRYPTO_SECRETBOX_MACBYTES,
	kdf::{Key, StackKdf},
	types::ByteArray,
};
use dsnp_graph_config::errors::{DsnpGraphError, DsnpGraphResult};
use log::Level;
use log_result_proc_macro::log_result_err;
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
		a_secret_key: &SecretKeyType,
		b_public_key: &PublicKeyType,
	) -> DsnpGraphResult<Self::DsnpPrid>;

	/// creates shared context from A -> B
	fn create_shared_context(
		b: DsnpUserId,
		a_secret_key: &SecretKeyType,
		b_public_key: &PublicKeyType,
	) -> DsnpGraphResult<Key>;
}

impl PridProvider for DsnpPrid {
	type DsnpPrid = DsnpPrid;

	#[log_result_err(Level::Info)]
	fn create_prid(
		a: DsnpUserId,
		b: DsnpUserId,
		a_secret_key: &SecretKeyType,
		b_public_key: &PublicKeyType,
	) -> DsnpGraphResult<Self::DsnpPrid> {
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

	#[log_result_err(Level::Info)]
	fn create_shared_context(
		b: DsnpUserId,
		a_secret_key: &SecretKeyType,
		b_public_key: &PublicKeyType,
	) -> DsnpGraphResult<Key> {
		// calculate shared secret
		let root_shared = match (a_secret_key, b_public_key) {
			(SecretKeyType::Version1_0(a_pair), PublicKeyType::Version1_0(b_public)) => {
				Zeroizing::new(crypto_box_beforenm(
					b_public.as_array(),
					a_pair.secret_key.as_array(),
				))
			},
		};

		// // derive a new key form pri context
		let kdf =
			StackKdf::from_parts(Key::from(root_shared.deref()), PRI_CONTEXT.as_array().into());
		let derived_key: Key = kdf
			.derive_subkey(b)
			.map_err(|e| DsnpGraphError::KeyDerivationError(e.to_string()))?;

		Ok(derived_key)
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::dsnp::dsnp_types::DsnpPrid;
	use dryoc::{
		dryocsecretbox::Key,
		keypair::{PublicKey, SecretKey, StackKeyPair},
		types::ByteArray,
	};

	#[test]
	fn generated_pri_should_be_the_same_calculated_from_both_sides() {
		let a = 2576367222u64;
		let b = 826378782u64;
		let key_pair_a = StackKeyPair::gen();
		let key_pair_b = StackKeyPair::gen();

		let pri_a_to_b = DsnpPrid::create_prid(
			a,
			b,
			&SecretKeyType::Version1_0(key_pair_a.clone()),
			&PublicKeyType::Version1_0(key_pair_b.public_key.clone()),
		)
		.expect("should create pri");
		let pri_a_to_b_2 = DsnpPrid::create_prid(
			a,
			b,
			&SecretKeyType::Version1_0(key_pair_b),
			&PublicKeyType::Version1_0(key_pair_a.public_key.clone()),
		)
		.expect("should create pri");

		assert_eq!(pri_a_to_b, pri_a_to_b_2);
	}

	#[test]
	fn generated_prid_should_be_compatible_with_test_vector() {
		let alice = 42;
		let bob = 478;
		let alice_secret = SecretKey::try_from(
			hex::decode("c9432ed5c0c5c24e8a4ff190619893918b4d1265a67d123895023fa7324b43e0")
				.expect("should decode")
				.as_array(),
		)
		.unwrap();
		let alice_public = PublicKey::try_from(
			hex::decode("0fea2cafabdc83752be36fa5349640da2c828add0a290df13cd2d8173eb2496f")
				.expect("should decode")
				.as_array(),
		)
		.unwrap();
		let bob_secret = SecretKey::try_from(
			hex::decode("dc106e1371293ee9536956e1253f43f8941d4a5c4e40f15968d24b75512b6920")
				.expect("should decode")
				.as_array(),
		)
		.unwrap();
		let bob_public = PublicKey::try_from(
			hex::decode("d0d4eb21db1df63369c147e63b2573816dd4b3fe513e95bf87f7ed1835407e62")
				.expect("should decode")
				.as_array(),
		)
		.unwrap();
		let expected_alice_to_bob =
			DsnpPrid::new(&hex::decode("ace4d2995b1a829c").expect("should decode"));
		let expected_ctx_alice_to_bob = Key::from(
			hex::decode("37cb1a870f0c1dce06f5116faf145ac2cf7a2f7d30136be4eea70c324932e6d2")
				.expect("should decode")
				.as_array(),
		);
		let expected_bob_to_alice =
			DsnpPrid::new(&hex::decode("1a53b02a26503600").expect("should decode"));
		let expected_ctx_bob_to_alice = Key::from(
			hex::decode("32c45c49fcfe12f9db60e74fa66416c5a05832c298814d82032a6783a4b1fca0")
				.expect("should decode")
				.as_array(),
		);

		let pri_alice_to_bob = DsnpPrid::create_prid(
			alice,
			bob,
			&SecretKeyType::Version1_0(StackKeyPair::from_secret_key(alice_secret.clone())),
			&PublicKeyType::Version1_0(bob_public.clone()),
		)
		.expect("should create pri");
		let ctx_alice_to_bob = DsnpPrid::create_shared_context(
			bob,
			&&SecretKeyType::Version1_0(StackKeyPair::from_secret_key(alice_secret)),
			&PublicKeyType::Version1_0(bob_public),
		)
		.expect("should create ctx");

		let pri_bob_to_alice = DsnpPrid::create_prid(
			bob,
			alice,
			&SecretKeyType::Version1_0(StackKeyPair::from_secret_key(bob_secret.clone())),
			&PublicKeyType::Version1_0(alice_public.clone()),
		)
		.expect("should create pri");
		let ctx_bob_to_alice = DsnpPrid::create_shared_context(
			alice,
			&SecretKeyType::Version1_0(StackKeyPair::from_secret_key(bob_secret)),
			&PublicKeyType::Version1_0(alice_public),
		)
		.expect("should create ctx");

		assert_eq!(pri_alice_to_bob, expected_alice_to_bob);
		assert_eq!(ctx_alice_to_bob, expected_ctx_alice_to_bob);

		assert_eq!(pri_bob_to_alice, expected_bob_to_alice);
		assert_eq!(ctx_bob_to_alice, expected_ctx_bob_to_alice);
	}
}
