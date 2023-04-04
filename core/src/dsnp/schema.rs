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
