use apache_avro::Schema;
use lazy_static::lazy_static;

lazy_static! {
	static ref PUBLIC_KEY_SCHEMA: Schema =
		Schema::parse_str(include_str!("../../schemas/public_key_schema.json"))
			.expect("public_key_schema.json is not avro compatible");
	static ref PUBLIC_GRAPH_CHUNK_SCHEMA: Schema =
		Schema::parse_str(include_str!("../../schemas/user_public_graph_chunk.json"))
			.expect("user_public_graph_chunk.json is not avro compatible");
	static ref PUBLIC_GRAPH_SCHEMA: Schema =
		Schema::parse_str(include_str!("../../schemas/public_graph.json"))
			.expect("public_graph.json is not avro compatible");
	static ref PRIVATE_GRAPH_CHUNK_SCHEMA: Schema =
		Schema::parse_str(include_str!("../../schemas/user_private_graph_chunk.json"))
			.expect("user_private_graph_chunk.json is not avro compatible");
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::types::*;
	use apache_avro::{from_avro_datum, from_value, to_avro_datum, to_value};

	#[test]
	fn should_successfully_read_and_write_public_key_from_and_to_schema() {
		let key = PublicKey {
			key_id: 128,
			revoked_as_of: 1187238222,
			key: b"217678127812871812334324".to_vec(),
		};

		let val = to_value(key.clone()).expect("should convert to value");
		let datum = to_avro_datum(&PUBLIC_KEY_SCHEMA, val).expect("should write datum");

		// parse input binary to schema
		let reader = from_avro_datum(&PUBLIC_KEY_SCHEMA, &mut &datum[..], None)
			.expect("should read from datum");
		let deserialized = from_value::<PublicKey>(&reader).expect("should deserialize");
		assert_eq!(deserialized, key);
	}

	#[test]
	fn should_successfully_read_and_write_public_graph_chunk_from_and_to_schema() {
		let chunk = UserPublicGraphChunk {
			compressed_public_graph: b"shugdua781262876euwsdgjdgjay981613789y1278eywhgdjhs"
				.to_vec(),
		};

		let val = to_value(chunk.clone()).expect("should convert to value");
		let datum = to_avro_datum(&PUBLIC_GRAPH_CHUNK_SCHEMA, val).expect("should write datum");

		// parse input binary to schema
		let reader = from_avro_datum(&PUBLIC_GRAPH_CHUNK_SCHEMA, &mut &datum[..], None)
			.expect("should read from datum");
		let deserialized = from_value::<UserPublicGraphChunk>(&reader).expect("should deserialize");
		assert_eq!(deserialized, chunk);
	}

	#[test]
	fn should_successfully_read_and_write_public_graph_from_and_to_schema() {
		let public_graph: PublicGraph = vec![
			GraphEdge { user_id: 7, since: 12638718 },
			GraphEdge { user_id: 167282, since: 28638718 },
		];

		let val = to_value(public_graph.clone()).expect("should convert to value");
		let datum = to_avro_datum(&PUBLIC_GRAPH_SCHEMA, val).expect("should write datum");

		// parse input binary to schema
		let reader = from_avro_datum(&PUBLIC_GRAPH_SCHEMA, &mut &datum[..], None)
			.expect("should read from datum");
		let deserialized = from_value::<PublicGraph>(&reader).expect("should deserialize");
		assert_eq!(deserialized, public_graph);
	}

	#[test]
	fn should_successfully_read_and_write_private_graph_chunk_from_and_to_schema() {
		let chunk = UserPrivateGraphChunk {
			encrypted_compressed_private_graph:
				b"shugdua781262876euwsdgjdgjay981613789y1278eywhgdjhs".to_vec(),
			key_id: 26783,
			prids: vec![
				Prid::new(27737272u64.to_le_bytes().as_slice()),
				Prid::new(17237271u64.to_le_bytes().as_slice()),
			],
		};

		let val = to_value(chunk.clone()).expect("should convert to value");
		let datum = to_avro_datum(&PRIVATE_GRAPH_CHUNK_SCHEMA, val).expect("should write datum");

		// parse input binary to schema
		let reader = from_avro_datum(&PRIVATE_GRAPH_CHUNK_SCHEMA, &mut &datum[..], None)
			.expect("should read from datum");
		let deserialized =
			from_value::<UserPrivateGraphChunk>(&reader).expect("should deserialize");
		assert_eq!(deserialized, chunk);
	}
}
