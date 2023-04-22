use crate::dsnp::encryption::{EncryptionBehavior, SealBox};
use dryoc::keypair::{PublicKey, StackKeyPair};
use dsnp_graph_config::DsnpVersion;

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
	pub fn new(version: DsnpVersion) -> Self {
		match version {
			DsnpVersion::Version1_0 => DsnpVersionConfig::Version1_0 { algorithm: SealBox },
		}
	}

	pub fn get_algorithm(&self) -> Box<dyn EncryptionBehavior> {
		match self {
			DsnpVersionConfig::Version1_0 { algorithm } => Box::new(algorithm.clone()),
		}
	}
}

impl KeyPairType {
	pub fn get_public_key_raw(&self) -> Vec<u8> {
		match self {
			KeyPairType::Version1_0(k) => k.public_key.to_vec(),
		}
	}
}

impl Into<PublicKeyType> for &KeyPairType {
	fn into(self) -> PublicKeyType {
		match self {
			KeyPairType::Version1_0(k) => PublicKeyType::Version1_0(k.clone().public_key),
		}
	}
}

impl Into<SecretKeyType> for KeyPairType {
	fn into(self) -> SecretKeyType {
		match self {
			KeyPairType::Version1_0(k) => SecretKeyType::Version1_0(k),
		}
	}
}

impl Into<PublicKeyType> for KeyPairType {
	fn into(self) -> PublicKeyType {
		match self {
			KeyPairType::Version1_0(k) => PublicKeyType::Version1_0(k.public_key),
		}
	}
}

impl Into<DsnpVersionConfig> for &SecretKeyType {
	fn into(self) -> DsnpVersionConfig {
		match self {
			SecretKeyType::Version1_0(_) => DsnpVersionConfig::new(DsnpVersion::Version1_0),
		}
	}
}

impl Into<DsnpVersionConfig> for &KeyPairType {
	fn into(self) -> DsnpVersionConfig {
		match self {
			KeyPairType::Version1_0(_) => DsnpVersionConfig::new(DsnpVersion::Version1_0),
		}
	}
}

impl Into<DsnpVersionConfig> for &PublicKeyType {
	fn into(self) -> DsnpVersionConfig {
		match self {
			PublicKeyType::Version1_0(_) => DsnpVersionConfig::new(DsnpVersion::Version1_0),
		}
	}
}

impl Into<Vec<u8>> for PublicKeyType {
	fn into(self) -> Vec<u8> {
		match self {
			PublicKeyType::Version1_0(k) => k.to_vec(),
		}
	}
}
