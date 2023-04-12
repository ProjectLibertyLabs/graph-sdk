use super::*;
use crate::{
	dsnp::{
		compression::{CompressionBehavior, DeflateCompression},
		dsnp_types::{
			DsnpInnerGraph, DsnpPublicKey, DsnpUserPrivateGraphChunk, DsnpUserPublicGraphChunk,
		},
		encryption::{EncryptionBehavior, SealBox},
		reader_writer::{DsnpBase, DsnpReader, DsnpWriter},
		schema::SchemaHandler,
	},
	types::PrivateGraphChunk,
};
use anyhow::Result;

/// DsnpBase implementation for Frequency
impl DsnpBase for Frequency {
	/// using SealBox for encryption and decryption
	type Encryption = SealBox;
}

/// implementing DsnpReader for Frequency
impl DsnpReader for Frequency {
	fn read_public_key(data: &[u8]) -> Result<DsnpPublicKey> {
		SchemaHandler::read_public_key(data)
	}

	fn read_public_graph(data: &[u8]) -> Result<DsnpInnerGraph> {
		let chunk = SchemaHandler::read_public_graph_chunk(data)?;
		let decompressed = DeflateCompression::decompress(&chunk.compressed_public_graph)?;
		SchemaHandler::read_inner_graph(&decompressed)
	}

	fn read_private_graph(
		data: &[u8],
		decryption_input: &<Self::Encryption as EncryptionBehavior>::DecryptionInput,
	) -> Result<PrivateGraphChunk> {
		let chunk = SchemaHandler::read_private_graph_chunk(data)?;
		let decrypted_compressed =
			Self::Encryption::decrypt(&chunk.encrypted_compressed_private_graph, decryption_input)?;
		let decompressed = DeflateCompression::decompress(&decrypted_compressed)?;
		Ok(PrivateGraphChunk {
			prids: chunk.prids,
			key_id: chunk.key_id,
			inner_graph: SchemaHandler::read_inner_graph(&decompressed)?,
		})
	}
}

/// implementing DsnpWriter for Frequency
impl DsnpWriter for Frequency {
	fn write_public_key(key: &DsnpPublicKey) -> Result<Vec<u8>> {
		SchemaHandler::write_public_key(key)
	}

	fn write_public_graph(inner: &DsnpInnerGraph) -> Result<Vec<u8>> {
		let serialized = SchemaHandler::write_inner_graph(inner)?;
		let compressed_public_graph = DeflateCompression::compress(&serialized)?;
		SchemaHandler::write_public_graph_chunk(&DsnpUserPublicGraphChunk {
			compressed_public_graph,
		})
	}

	fn write_private_graph(
		graph: &PrivateGraphChunk,
		encryption_input: &<Self::Encryption as EncryptionBehavior>::EncryptionInput,
	) -> Result<Vec<u8>> {
		let inner_serialized = SchemaHandler::write_inner_graph(&graph.inner_graph)?;
		let compressed_inner = DeflateCompression::compress(&inner_serialized)?;
		let encrypted_compressed = Self::Encryption::encrypt(&compressed_inner, encryption_input)?;
		SchemaHandler::write_private_graph_chunk(&DsnpUserPrivateGraphChunk {
			key_id: graph.key_id,
			prids: graph.prids.to_owned(),
			encrypted_compressed_private_graph: encrypted_compressed,
		})
	}
}
