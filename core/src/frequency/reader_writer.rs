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

#[cfg(test)]
mod tests {
	use super::*;
	use crate::dsnp::dsnp_types::{DsnpGraphEdge, DsnpPrid};
	use dryoc::keypair::StackKeyPair;
	use rand::Rng;

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
		let key_pair = StackKeyPair::gen();

		let serialized = Frequency::write_private_graph(&private_graph, &key_pair.public_key)
			.expect("serialization should work");
		let deserialized = Frequency::read_private_graph(&serialized, &key_pair)
			.expect("deserialization should work");

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
		let key_pair = StackKeyPair::gen();

		let mut serialized = Frequency::write_private_graph(&private_graph, &key_pair.public_key)
			.expect("serialization should work");
		serialized.pop(); // corrupting the input
		let deserialized = Frequency::read_private_graph(&serialized, &key_pair);

		assert!(deserialized.is_err());
	}

	#[test]
	fn check_average_size_of_graph_page() {
		let mut rng = rand::thread_rng();

		let connections = 300;
		let page_size = 1 << 11;
		let mut inner_graph: DsnpInnerGraph = vec![];
		let mut prids = vec![];
		for i in 0..connections {
			inner_graph.push(DsnpGraphEdge {
				user_id: rng.gen_range(1..(u64::MAX / 2)),
				since: (1679604427 + i),
			});
			let pri: [u8; 8] = rng.gen();
			prids.push(DsnpPrid::new(&pri));
		}

		let public_serialized =
			Frequency::write_public_graph(&inner_graph).expect("serialization should work");

		let private_graph = PrivateGraphChunk { inner_graph, key_id: 200, prids };
		let key_pair = StackKeyPair::gen();
		let private_serialized =
			Frequency::write_private_graph(&private_graph, &key_pair.public_key)
				.expect("serialization should work");

		println!(
			"public graph: size of {} connections in a page is {}",
			connections,
			public_serialized.len()
		);
		println!(
			"private graph: size of {} connections in a page is {}",
			connections,
			private_serialized.len()
		);

		assert_eq!((public_serialized.len() - 1) / page_size + 1, 2);
		assert_eq!((private_serialized.len() - 1) / page_size + 1, 3);
	}
}
