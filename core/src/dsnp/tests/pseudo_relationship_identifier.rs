use crate::dsnp::{dsnp_types::DsnpPrid, pseudo_relationship_identifier::PridProvider};
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

	let pri_a_to_b = DsnpPrid::create_prid(a, b, &key_pair_a.secret_key, &key_pair_b.public_key)
		.expect("should create pri");
	let pri_a_to_b_2 = DsnpPrid::create_prid(a, b, &key_pair_b.secret_key, &key_pair_a.public_key)
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

	let pri_alice_to_bob =
		DsnpPrid::create_prid(alice, bob, &&alice_secret, &bob_public).expect("should create pri");
	let ctx_alice_to_bob = DsnpPrid::create_shared_context(bob, &&alice_secret, &bob_public)
		.expect("should create ctx");

	let pri_bob_to_alice =
		DsnpPrid::create_prid(bob, alice, &bob_secret, &alice_public).expect("should create pri");
	let ctx_bob_to_alice = DsnpPrid::create_shared_context(alice, &&bob_secret, &alice_public)
		.expect("should create ctx");

	assert_eq!(pri_alice_to_bob, expected_alice_to_bob);
	assert_eq!(ctx_alice_to_bob, expected_ctx_alice_to_bob);

	assert_eq!(pri_bob_to_alice, expected_bob_to_alice);
	assert_eq!(ctx_bob_to_alice, expected_ctx_bob_to_alice);
}
