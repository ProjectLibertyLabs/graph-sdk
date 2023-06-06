//! DSNP version specific algorithms and Key types
use crate::{
	api::api_types::GraphKeyPair,
	dsnp::{
		dsnp_types::DsnpPublicKey,
		encryption::{EncryptionBehavior, SealBox},
	},
};
use dryoc::keypair::{PublicKey, SecretKey, StackKeyPair};
use dsnp_graph_config::{
	errors::{DsnpGraphError, DsnpGraphResult},
	DsnpVersion, GraphKeyType,
};

/// Dsnp versions hardcoded configuration
#[derive(Clone, PartialEq, Debug, Eq, Hash)]
pub enum DsnpVersionConfig {
	/// Dsnp version 1.0
	Version1_0 { algorithm: SealBox },
}

/// Public key types for dsnp versions
#[derive(Clone, PartialEq, Debug)]
pub enum PublicKeyType {
	/// Dsnp version 1.0
	Version1_0(PublicKey),
}

/// Keypair types for dsnp versions
#[derive(Clone, PartialEq, Debug)]
pub enum KeyPairType {
	/// Dsnp version 1.0
	Version1_0(StackKeyPair),
}

/// Secret key types for dsnp versions
#[derive(Clone, PartialEq, Debug)]
pub enum SecretKeyType {
	/// Dsnp version 1.0
	Version1_0(StackKeyPair),
}

impl DsnpVersionConfig {
	/// creates a new `DsnpVersionConfig` based on the version enum
	pub fn new(version: DsnpVersion) -> Self {
		match version {
			DsnpVersion::Version1_0 => DsnpVersionConfig::Version1_0 { algorithm: SealBox },
		}
	}

	/// returns the encryption/description algorithm associated with dsnp version
	pub fn get_algorithm(&self) -> Box<dyn EncryptionBehavior> {
		match self {
			DsnpVersionConfig::Version1_0 { algorithm } => Box::new(algorithm.clone()),
		}
	}
}

impl KeyPairType {
	/// returns raw bytes of the public key for specified dsnp version
	pub fn get_public_key_raw(&self) -> Vec<u8> {
		match self {
			KeyPairType::Version1_0(k) => k.public_key.to_vec(),
		}
	}

	#[cfg(test)]
	/// returns raw bytes of the secret key for specified dsnp version
	pub fn get_secret_key_raw(&self) -> Vec<u8> {
		match self {
			KeyPairType::Version1_0(k) => k.secret_key.to_vec(),
		}
	}
}

/// converts a reference of `KeyPairType` into a `PublicKeyType`
impl Into<PublicKeyType> for &'_ KeyPairType {
	fn into(self) -> PublicKeyType {
		match self {
			KeyPairType::Version1_0(k) => PublicKeyType::Version1_0(k.clone().public_key),
		}
	}
}

/// converts a `KeyPairType` into a `SecretKeyType`
impl Into<SecretKeyType> for KeyPairType {
	fn into(self) -> SecretKeyType {
		match self {
			KeyPairType::Version1_0(k) => SecretKeyType::Version1_0(k),
		}
	}
}

/// converts a `SecretKeyType` into a `DsnpVersionConfig`
impl Into<DsnpVersionConfig> for &SecretKeyType {
	fn into(self) -> DsnpVersionConfig {
		match self {
			SecretKeyType::Version1_0(_) => DsnpVersionConfig::new(DsnpVersion::Version1_0),
		}
	}
}

/// converts a `KeyPairType` into a `DsnpVersionConfig`
impl Into<DsnpVersionConfig> for &KeyPairType {
	fn into(self) -> DsnpVersionConfig {
		match self {
			KeyPairType::Version1_0(_) => DsnpVersionConfig::new(DsnpVersion::Version1_0),
		}
	}
}

/// converts a `PublicKeyType` into a `DsnpVersionConfig`
impl Into<DsnpVersionConfig> for &PublicKeyType {
	fn into(self) -> DsnpVersionConfig {
		match self {
			PublicKeyType::Version1_0(_) => DsnpVersionConfig::new(DsnpVersion::Version1_0),
		}
	}
}

impl TryInto<PublicKeyType> for &'_ DsnpPublicKey {
	type Error = DsnpGraphError;

	fn try_into(self) -> DsnpGraphResult<PublicKeyType> {
		let public_key =
			PublicKey::try_from(&self.key[..]).map_err(|_| DsnpGraphError::InvalidPublicKey)?;
		Ok(PublicKeyType::Version1_0(public_key))
	}
}

/// converts a `PublicKeyType` into a `Vec<u8>`
impl Into<Vec<u8>> for PublicKeyType {
	fn into(self) -> Vec<u8> {
		match self {
			PublicKeyType::Version1_0(k) => k.to_vec(),
		}
	}
}

/// converts a `GraphKeyType` into a `KeyPairType`
impl TryInto<KeyPairType> for GraphKeyPair {
	type Error = DsnpGraphError;

	fn try_into(self) -> DsnpGraphResult<KeyPairType> {
		match self.key_type {
			GraphKeyType::X25519 => {
				let secret_key = SecretKey::try_from(&self.secret_key[..])
					.map_err(|_| DsnpGraphError::InvalidSecretKey)?;
				let pair = StackKeyPair::from_secret_key(secret_key);
				if pair.public_key.to_vec() != self.public_key {
					return Err(DsnpGraphError::PublicKeyNotCompatibleWithSecretKey)
				}
				Ok(KeyPairType::Version1_0(pair))
			},
		}
	}
}
