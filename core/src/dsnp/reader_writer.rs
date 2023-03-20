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

pub trait DsnpReader {
	/// reading public key from binary
	fn read_public_key(data: &[u8]) -> Result<DsnpPublicKey>;
	/// reading public graph from binary
	fn read_public_graph(data: &[u8]) -> Result<DsnpInnerGraph>;
	/// reading private graph from binary
	fn read_private_graph(data: &[u8]) -> Result<PrivateGraphChunk>;
}

pub trait DsnpWriter {
	/// write public key to binary
	fn write_public_key(key: &DsnpPublicKey) -> Result<Vec<u8>>;
	/// write public graph to binary
	fn write_public_graph(inner: &DsnpInnerGraph) -> Result<Vec<u8>>;
	/// write private graph to binary
	fn write_private_graph(graph: &PrivateGraphChunk) -> Result<Vec<u8>>;
}

/// A utility to read/write data from and to Frequency chain specific implementation of DSNP
pub struct Frequency;

impl DsnpReader for Frequency {
	fn read_public_key(data: &[u8]) -> Result<DsnpPublicKey> {
		SchemaHandler::read_public_key(data)
	}

	fn read_public_graph(data: &[u8]) -> Result<DsnpInnerGraph> {
		let chunk = SchemaHandler::read_public_graph_chunk(data)?;
		let decompressed = DeflateCompression::decompress(&chunk.compressed_public_graph)?;
		SchemaHandler::read_inner_graph(&decompressed)
	}

	fn read_private_graph(data: &[u8]) -> Result<PrivateGraphChunk> {
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
}

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

	fn write_private_graph(graph: &PrivateGraphChunk) -> Result<Vec<u8>> {
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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::dsnp::dsnp_types::{DsnpGraphEdge, DsnpPrid};

	#[test]
	fn public_graph_read_and_write_using_valid_input_should_succeed() {
		let inner_graph: DsnpInnerGraph = vec![
			DsnpGraphEdge { user_id: 7, since: 12638718 },
			DsnpGraphEdge { user_id: 167282, since: 28638718 },
		];

		let serialized =
			Frequency::write_public_graph(&inner_graph).expect("serialization should work");
		let deserialized =
			Frequency::read_public_graph(&serialized).expect("deserialization should work");

		assert_eq!(deserialized, inner_graph);
	}

	#[test]
	fn public_graph_read_using_invalid_input_should_fail() {
		let inner_graph: DsnpInnerGraph = vec![
			DsnpGraphEdge { user_id: 7, since: 12638718 },
			DsnpGraphEdge { user_id: 167282, since: 28638718 },
		];

		let mut serialized =
			Frequency::write_public_graph(&inner_graph).expect("serialization should work");
		serialized.pop(); // corrupting the input
		let deserialized = Frequency::read_public_graph(&serialized);

		assert!(deserialized.is_err());
	}

	#[test]
	fn private_graph_read_and_write_using_valid_input_should_succeed() {
		let private_graph = PrivateGraphChunk {
			inner_graph: vec![
				DsnpGraphEdge { user_id: 7, since: 12638718 },
				DsnpGraphEdge { user_id: 167282, since: 28638718 },
			],
			key_id: 26783,
			prids: vec![
				DsnpPrid::new(27737272u64.to_le_bytes().as_slice()),
				DsnpPrid::new(17237271u64.to_le_bytes().as_slice()),
			],
		};

		let serialized =
			Frequency::write_private_graph(&private_graph).expect("serialization should work");
		let deserialized =
			Frequency::read_private_graph(&serialized).expect("deserialization should work");

		assert_eq!(deserialized, private_graph);
	}

	#[test]
	fn private_graph_read_using_invalid_input_should_fail() {
		let private_graph = PrivateGraphChunk {
			inner_graph: vec![
				DsnpGraphEdge { user_id: 7, since: 12638718 },
				DsnpGraphEdge { user_id: 167282, since: 28638718 },
			],
			key_id: 26783,
			prids: vec![
				DsnpPrid::new(27737272u64.to_le_bytes().as_slice()),
				DsnpPrid::new(17237271u64.to_le_bytes().as_slice()),
			],
		};

		let mut serialized =
			Frequency::write_private_graph(&private_graph).expect("serialization should work");
		serialized.pop(); // corrupting the input
		let deserialized = Frequency::read_public_graph(&serialized);

		assert!(deserialized.is_err());
	}
}
