use crate::{
	dsnp::{
		compression::{CompressionBehavior, DeflateCompression},
		dsnp_types::{
			DsnpInnerGraph, DsnpPublicKey, DsnpUserPrivateGraphChunk, DsnpUserPublicGraphChunk,
		},
		schema::SchemaHandler,
	},
	types::PrivateGraphChunk,
};
use anyhow::Result;

pub struct FrequencyReaderWriter;

impl FrequencyReaderWriter {
	pub fn read_public_key(data: &[u8]) -> Result<DsnpPublicKey> {
		SchemaHandler::read_public_key(data)
	}

	pub fn write_public_key(key: &DsnpPublicKey) -> Result<Vec<u8>> {
		SchemaHandler::write_public_key(key)
	}

	pub fn read_public_graph(data: &[u8]) -> Result<DsnpInnerGraph> {
		let chunk = SchemaHandler::read_public_graph_chunk(data)?;
		let decompressed = DeflateCompression::decompress(&chunk.compressed_public_graph)?;
		SchemaHandler::read_inner_graph(&decompressed)
	}

	pub fn write_public_graph(inner: &DsnpInnerGraph) -> Result<Vec<u8>> {
		let serialized = SchemaHandler::write_inner_graph(inner)?;
		let compressed_public_graph = DeflateCompression::compress(&serialized)?;
		SchemaHandler::write_public_graph_chunk(&DsnpUserPublicGraphChunk {
			compressed_public_graph,
		})
	}

	pub fn read_private_graph(data: &[u8]) -> Result<PrivateGraphChunk> {
		let chunk = SchemaHandler::read_private_graph_chunk(data)?;
		// todo decrypt
		let decrypted_compressed = chunk.encrypted_compressed_private_graph;
		let decompressed = DeflateCompression::decompress(&decrypted_compressed)?;
		Ok(PrivateGraphChunk {
			prids: chunk.prids,
			key_id: chunk.key_id,
			inner_graph: SchemaHandler::read_inner_graph(&decompressed)?,
		})
	}

	pub fn write_private_graph(graph: &PrivateGraphChunk) -> Result<Vec<u8>> {
		let inner_serialized = SchemaHandler::write_inner_graph(&graph.inner_graph)?;
		let compressed_inner = DeflateCompression::compress(&inner_serialized)?;
		// todo encrypt
		let encrypted_compressed = compressed_inner;
		SchemaHandler::write_private_graph_chunk(&DsnpUserPrivateGraphChunk {
			key_id: graph.key_id,
			prids: graph.prids.to_owned(),
			encrypted_compressed_private_graph: encrypted_compressed,
		})
	}
}
