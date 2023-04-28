use crate::{
	dsnp::{
		dsnp_configs::{DsnpVersionConfig, PublicKeyType, SecretKeyType},
		dsnp_types::{DsnpInnerGraph, DsnpPublicKey},
		encryption::EncryptionBehavior,
	},
	types::PrivateGraphChunk,
};
use anyhow::Result;

/// a base trait to define common associated types
pub trait DsnpBase {
	type Encryption: EncryptionBehavior;
}

/// DSNP compatible reader
pub trait DsnpReader: DsnpBase {
	/// reading public key from binary
	fn read_public_key(data: &[u8]) -> Result<DsnpPublicKey>;
	/// reading public graph from binary
	fn read_public_graph(data: &[u8]) -> Result<DsnpInnerGraph>;
	/// reading private graph from binary
	fn read_private_graph(
		data: &[u8],
		dsnp_version_config: &DsnpVersionConfig,
		decryption_input: &SecretKeyType,
	) -> Result<PrivateGraphChunk>;
}

/// DSNP compatible writer
pub trait DsnpWriter: DsnpBase {
	/// write public key to binary
	fn write_public_key(key: &DsnpPublicKey) -> Result<Vec<u8>>;
	/// write public graph to binary
	fn write_public_graph(inner: &DsnpInnerGraph) -> Result<Vec<u8>>;
	/// write private graph to binary
	fn write_private_graph(
		graph: &PrivateGraphChunk,
		dsnp_version_config: &DsnpVersionConfig,
		encryption_input: &PublicKeyType,
	) -> Result<Vec<u8>>;
}
