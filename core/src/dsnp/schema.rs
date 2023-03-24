use crate::dsnp::dsnp_types::{
	DsnpInnerGraph, DsnpPublicKey, DsnpUserPrivateGraphChunk, DsnpUserPublicGraphChunk,
};
use anyhow::Result;
use apache_avro::{from_avro_datum, from_value, to_avro_datum, to_value, Schema};
use lazy_static::lazy_static;
use serde::{Deserialize, Serialize};

lazy_static! {
	static ref PUBLIC_KEY_SCHEMA: Schema =
		Schema::parse_str(include_str!("../../../schemas/public_key_schema.json")).unwrap();
	static ref PUBLIC_GRAPH_CHUNK_SCHEMA: Schema =
		Schema::parse_str(include_str!("../../../schemas/user_public_graph_chunk.json")).unwrap();
	static ref PUBLIC_GRAPH_SCHEMA: Schema =
		Schema::parse_str(include_str!("../../../schemas/public_graph.json")).unwrap();
	static ref PRIVATE_GRAPH_CHUNK_SCHEMA: Schema =
		Schema::parse_str(include_str!("../../../schemas/user_private_graph_chunk.json")).unwrap();
}

/// A utility to handle serialization and deserialization on specified schemas
pub struct SchemaHandler;

impl SchemaHandler {
	pub fn read_public_key(data: &[u8]) -> Result<DsnpPublicKey> {
		Self::read(data, &PUBLIC_KEY_SCHEMA)
	}

	pub fn write_public_key(key: &DsnpPublicKey) -> Result<Vec<u8>> {
		Self::write(key, &PUBLIC_KEY_SCHEMA)
	}

	pub fn read_public_graph_chunk(data: &[u8]) -> Result<DsnpUserPublicGraphChunk> {
		Self::read(data, &PUBLIC_GRAPH_CHUNK_SCHEMA)
	}

	pub fn write_public_graph_chunk(chunk: &DsnpUserPublicGraphChunk) -> Result<Vec<u8>> {
		Self::write(chunk, &PUBLIC_GRAPH_CHUNK_SCHEMA)
	}

	pub fn read_inner_graph(data: &[u8]) -> Result<DsnpInnerGraph> {
		Self::read(data, &PUBLIC_GRAPH_SCHEMA)
	}

	pub fn write_inner_graph(inner_graph: &DsnpInnerGraph) -> Result<Vec<u8>> {
		Self::write(inner_graph, &PUBLIC_GRAPH_SCHEMA)
	}

	pub fn read_private_graph_chunk(data: &[u8]) -> Result<DsnpUserPrivateGraphChunk> {
		Self::read(data, &PRIVATE_GRAPH_CHUNK_SCHEMA)
	}

	pub fn write_private_graph_chunk(chunk: &DsnpUserPrivateGraphChunk) -> Result<Vec<u8>> {
		Self::write(chunk, &PRIVATE_GRAPH_CHUNK_SCHEMA)
	}

	fn read<Output>(data: &[u8], schema: &Schema) -> Result<Output>
	where
		Output: for<'a> Deserialize<'a>,
	{
		let reader = from_avro_datum(schema, &mut &data[..], None)?;
		Ok(from_value::<Output>(&reader)?)
	}

	fn write<Input>(input: &Input, schema: &Schema) -> Result<Vec<u8>>
	where
		Input: Serialize,
	{
		let val = to_value(input)?;
		Ok(to_avro_datum(schema, val)?)
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::dsnp::dsnp_types::{DsnpGraphEdge, DsnpPrid};
	use apache_avro::Error as AvroError;

	#[test]
	fn public_key_read_and_write_using_valid_input_should_succeed() {
		let key = DsnpPublicKey {
			key_id: 128,
			revoked_as_of: 1187238222,
			key: b"217678127812871812334324".to_vec(),
		};

		let serialized = SchemaHandler::write_public_key(&key).expect("should serialize");
		let deserialized = SchemaHandler::read_public_key(&serialized).expect("should deserialize");

		assert_eq!(deserialized, key);
	}

	#[test]
	fn public_key_read_using_invalid_input_should_fail() {
		let key = DsnpPublicKey {
			key_id: 128,
			revoked_as_of: 1187238222,
			key: b"217678127812871812334324".to_vec(),
		};

		let mut serialized = SchemaHandler::write_public_key(&key).expect("should serialize");
		serialized[0] = serialized[0].saturating_add(1); // corrupting the input
		let deserialized = SchemaHandler::read_public_key(&serialized);

		assert!(deserialized.is_err());
		assert!(matches!(
			deserialized.unwrap_err().downcast_ref::<AvroError>(),
			Some(AvroError::ConvertI64ToUsize(_, _))
		))
	}

	#[test]
	fn public_key_read_using_input_bigger_than_i64_should_fail() {
		let key = DsnpPublicKey {
			key_id: 128,
			revoked_as_of: i64::MAX as u64 + 1,
			key: b"217678127812871812334324".to_vec(),
		};

		let serialized = SchemaHandler::write_public_key(&key);

		assert!(serialized.is_err());
		assert!(matches!(
			serialized.unwrap_err().downcast_ref::<AvroError>(),
			Some(AvroError::SerializeValue(_))
		))
	}

	#[test]
	fn public_graph_chunk_read_and_write_using_valid_input_should_succeed() {
		let chunk = DsnpUserPublicGraphChunk {
			compressed_public_graph: b"shugdua781262876euwsdgjdgjay981613789y1278eywhgdjhs"
				.to_vec(),
		};

		let serialized = SchemaHandler::write_public_graph_chunk(&chunk).expect("should serialize");
		let deserialized =
			SchemaHandler::read_public_graph_chunk(&serialized).expect("should deserialize");

		assert_eq!(deserialized, chunk);
	}

	#[test]
	fn inner_graph_read_and_write_using_valid_input_should_succeed() {
		let inner_graph: DsnpInnerGraph = vec![
			DsnpGraphEdge { user_id: 7, since: 12638718 },
			DsnpGraphEdge { user_id: 167282, since: 28638718 },
		];

		let serialized = SchemaHandler::write_inner_graph(&inner_graph).expect("should serialize");
		let deserialized =
			SchemaHandler::read_inner_graph(&serialized).expect("should deserialize");

		assert_eq!(deserialized, inner_graph);
	}

	#[test]
	fn private_graph_chunk_read_and_write_using_valid_input_should_succeed() {
		let chunk = DsnpUserPrivateGraphChunk {
			encrypted_compressed_private_graph:
				b"shugdua781262876euwsdgjdgjay981613789y1278eywhgdjhs".to_vec(),
			key_id: 26783,
			prids: vec![
				DsnpPrid::new(27737272u64.to_le_bytes().as_slice()),
				DsnpPrid::new(17237271u64.to_le_bytes().as_slice()),
			],
		};

		let serialized =
			SchemaHandler::write_private_graph_chunk(&chunk).expect("should serialize");
		let deserialized =
			SchemaHandler::read_private_graph_chunk(&serialized).expect("should deserialize");

		assert_eq!(deserialized, chunk);
	}
}
