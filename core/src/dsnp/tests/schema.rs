use crate::dsnp::{
	dsnp_types::{
		DsnpGraphEdge, DsnpInnerGraph, DsnpPrid, DsnpPublicKey, DsnpUserPrivateGraphChunk,
		DsnpUserPublicGraphChunk,
	},
	schema::SchemaHandler,
};
use apache_avro::Error as AvroError;

#[test]
fn public_key_read_and_write_using_valid_input_should_succeed() {
	let key = DsnpPublicKey { key_id: 128, key: b"217678127812871812334324".to_vec() };

	let serialized = SchemaHandler::write_public_key(&key).expect("should serialize");
	let deserialized = SchemaHandler::read_public_key(&serialized).expect("should deserialize");

	assert_eq!(deserialized, key);
}

#[test]
fn public_key_read_using_invalid_input_should_fail() {
	let key = DsnpPublicKey { key_id: 128, key: b"217678127812871812334324".to_vec() };

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
fn public_graph_chunk_read_and_write_using_valid_input_should_succeed() {
	let chunk = DsnpUserPublicGraphChunk {
		compressed_public_graph: b"shugdua781262876euwsdgjdgjay981613789y1278eywhgdjhs".to_vec(),
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
	let deserialized = SchemaHandler::read_inner_graph(&serialized).expect("should deserialize");

	assert_eq!(deserialized, inner_graph);
}

#[test]
fn private_graph_chunk_read_and_write_using_valid_input_should_succeed() {
	let chunk = DsnpUserPrivateGraphChunk {
		encrypted_compressed_private_graph: b"shugdua781262876euwsdgjdgjay981613789y1278eywhgdjhs"
			.to_vec(),
		key_id: 26783,
		prids: vec![
			DsnpPrid::new(27737272u64.to_le_bytes().as_slice()),
			DsnpPrid::new(17237271u64.to_le_bytes().as_slice()),
		],
	};

	let serialized = SchemaHandler::write_private_graph_chunk(&chunk).expect("should serialize");
	let deserialized =
		SchemaHandler::read_private_graph_chunk(&serialized).expect("should deserialize");

	assert_eq!(deserialized, chunk);
}
