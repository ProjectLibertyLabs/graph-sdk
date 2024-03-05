use super::*;
use crate::dsnp::{
	compression::{CompressionBehavior, DeflateCompression},
	dsnp_configs::{DsnpVersionConfig, PublicKeyType, SecretKeyType},
	dsnp_types::{
		DsnpInnerGraph, DsnpPublicKey, DsnpUserPrivateGraphChunk, DsnpUserPublicGraphChunk,
		PrivateGraphChunk,
	},
	reader_writer::{DsnpReader, DsnpWriter},
	schema::SchemaHandler,
};
use dsnp_graph_config::errors::DsnpGraphResult;
use log::Level;
use log_result_proc_macro::log_result_err;

/// implementing DsnpReader for Frequency
impl DsnpReader for Frequency {
	#[log_result_err(Level::Info)]
	fn read_public_key(data: &[u8]) -> DsnpGraphResult<DsnpPublicKey> {
		SchemaHandler::read_public_key(data)
	}

	fn read_public_graph(data: &[u8]) -> DsnpGraphResult<DsnpInnerGraph> {
		let chunk = SchemaHandler::read_public_graph_chunk(data)?;
		let decompressed = DeflateCompression::decompress(&chunk.compressed_public_graph)?;
		SchemaHandler::read_inner_graph(&decompressed)
	}

	fn read_private_graph(
		data: &[u8],
		dsnp_version_config: &DsnpVersionConfig,
		decryption_input: &SecretKeyType,
	) -> DsnpGraphResult<PrivateGraphChunk> {
		let chunk = SchemaHandler::read_private_graph_chunk(data)?;
		let decrypted_compressed = dsnp_version_config
			.get_algorithm()
			.decrypt(&chunk.encrypted_compressed_private_graph, decryption_input)?;
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
	fn write_public_key(key: &DsnpPublicKey) -> DsnpGraphResult<Vec<u8>> {
		SchemaHandler::write_public_key(key)
	}

	fn write_public_graph(inner: &DsnpInnerGraph) -> DsnpGraphResult<Vec<u8>> {
		let serialized = SchemaHandler::write_inner_graph(inner)?;
		let compressed_public_graph = DeflateCompression::compress(&serialized)?;
		SchemaHandler::write_public_graph_chunk(&DsnpUserPublicGraphChunk {
			compressed_public_graph,
		})
	}

	fn write_private_graph(
		graph: &PrivateGraphChunk,
		dsnp_version_config: &DsnpVersionConfig,
		encryption_input: &PublicKeyType,
	) -> DsnpGraphResult<Vec<u8>> {
		let inner_serialized = SchemaHandler::write_inner_graph(&graph.inner_graph)?;
		let compressed_inner = DeflateCompression::compress(&inner_serialized)?;
		let encrypted_compressed = dsnp_version_config
			.get_algorithm()
			.encrypt(&compressed_inner, encryption_input)?;
		SchemaHandler::write_private_graph_chunk(&DsnpUserPrivateGraphChunk {
			key_id: graph.key_id,
			prids: graph.prids.to_owned(),
			encrypted_compressed_private_graph: encrypted_compressed,
		})
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::dsnp::{
		dsnp_configs::KeyPairType,
		dsnp_types::{DsnpGraphEdge, DsnpPrid},
		encryption::SealBox,
	};
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
		let key_pair = KeyPairType::Version1_0(StackKeyPair::gen());

		let serialized = Frequency::write_private_graph(
			&private_graph,
			&DsnpVersionConfig::Version1_0 { algorithm: SealBox },
			&(&key_pair).into(),
		)
		.expect("serialization should work");
		let deserialized = Frequency::read_private_graph(
			&serialized,
			&DsnpVersionConfig::Version1_0 { algorithm: SealBox },
			&key_pair.into(),
		)
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
		let key_pair = KeyPairType::Version1_0(StackKeyPair::gen());

		let mut serialized = Frequency::write_private_graph(
			&private_graph,
			&DsnpVersionConfig::Version1_0 { algorithm: SealBox },
			&(&key_pair).into(),
		)
		.expect("serialization should work");
		serialized.pop(); // corrupting the input
		let deserialized = Frequency::read_private_graph(
			&serialized,
			&DsnpVersionConfig::Version1_0 { algorithm: SealBox },
			&key_pair.into(),
		);

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
		let key_pair = KeyPairType::Version1_0(StackKeyPair::gen());
		let private_serialized = Frequency::write_private_graph(
			&private_graph,
			&DsnpVersionConfig::Version1_0 { algorithm: SealBox },
			&(&key_pair).into(),
		)
		.expect("serialization should work");

		assert_eq!((public_serialized.len() - 1) / page_size + 1, 2);
		assert_eq!((private_serialized.len() - 1) / page_size + 1, 3);
	}
}
