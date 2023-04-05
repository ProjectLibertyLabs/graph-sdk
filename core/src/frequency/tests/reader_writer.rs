use crate::{
	dsnp::{
		dsnp_types::{DsnpGraphEdge, DsnpInnerGraph, DsnpPrid},
		reader_writer::{DsnpReader, DsnpWriter},
	},
	frequency::Frequency,
	types::PrivateGraphChunk,
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
		last_updated: 1234567,
	};
	let key_pair = StackKeyPair::gen();

	let serialized = Frequency::write_private_graph(&private_graph, &key_pair.public_key)
		.expect("serialization should work");
	let deserialized =
		Frequency::read_private_graph(&serialized, &key_pair).expect("deserialization should work");

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
		last_updated: 1234567,
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

	let private_graph =
		PrivateGraphChunk { inner_graph, key_id: 200, prids, last_updated: 1234567 };
	let key_pair = StackKeyPair::gen();
	let private_serialized = Frequency::write_private_graph(&private_graph, &key_pair.public_key)
		.expect("serialization should work");

	assert_eq!((public_serialized.len() - 1) / page_size + 1, 2);
	assert_eq!((private_serialized.len() - 1) / page_size + 1, 3);
}
