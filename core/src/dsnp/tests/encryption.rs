use crate::dsnp::encryption::{EncryptionBehavior, SealBox};
use dryoc::keypair::StackKeyPair;

#[test]
fn sealbox_should_encrypt_and_decrypt_successfully() {
	let plain_data = vec![
		23, 23, 109, 198, 111, 70, 2, 89, 2u8, 1, 0, 23, 5, 82, 100, 56, 1, 120, 200, 250, 140, 83,
		98, 0, 10, 234, 88, 23, 54, 23, 23, 109, 198, 111, 70, 2, 89,
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
