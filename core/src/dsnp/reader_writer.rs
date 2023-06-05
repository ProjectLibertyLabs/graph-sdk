use crate::dsnp::{
	dsnp_configs::{DsnpVersionConfig, PublicKeyType, SecretKeyType},
	dsnp_types::{DsnpInnerGraph, DsnpPublicKey, PrivateGraphChunk},
};
use dsnp_graph_config::errors::DsnpGraphResult;

/// DSNP compatible reader
pub trait DsnpReader {
	/// reading public key from binary
	fn read_public_key(data: &[u8]) -> DsnpGraphResult<DsnpPublicKey>;
	/// reading public graph from binary
	fn read_public_graph(data: &[u8]) -> DsnpGraphResult<DsnpInnerGraph>;
	/// reading private graph from binary
	fn read_private_graph(
		data: &[u8],
		dsnp_version_config: &DsnpVersionConfig,
		decryption_input: &SecretKeyType,
	) -> DsnpGraphResult<PrivateGraphChunk>;
}

/// DSNP compatible writer
pub trait DsnpWriter {
	/// write public key to binary
	fn write_public_key(key: &DsnpPublicKey) -> DsnpGraphResult<Vec<u8>>;
	/// write public graph to binary
	fn write_public_graph(inner: &DsnpInnerGraph) -> DsnpGraphResult<Vec<u8>>;
	/// write private graph to binary
	fn write_private_graph(
		graph: &PrivateGraphChunk,
		dsnp_version_config: &DsnpVersionConfig,
		encryption_input: &PublicKeyType,
	) -> DsnpGraphResult<Vec<u8>>;
}
