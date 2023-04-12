#![allow(dead_code)]
use crate::{
	dsnp::{
		api_types::*,
		compression::{CompressionBehavior, DeflateCompression},
		dsnp_types::*,
		encryption::EncryptionBehavior,
		schema::SchemaHandler,
	},
	graph::{graph::Graph, page::GraphPage},
	util::time::time_in_ksecs,
};

use base64::{engine::general_purpose, Engine as _};

pub fn create_graph_edge(id: &DsnpUserId) -> DsnpGraphEdge {
	DsnpGraphEdge { user_id: *id, since: time_in_ksecs() }
}

impl From<DsnpUserId> for DsnpPrid {
	fn from(id: DsnpUserId) -> Self {
		Self::from(id.to_le_bytes().to_vec())
	}
}

pub fn create_page(ids: &[DsnpUserId]) -> GraphPage {
	let mut page = GraphPage::new(crate::dsnp::api_types::PrivacyType::Private, 0);
	page.set_connections(ids.iter().map(create_graph_edge).collect());
	page.set_prids(ids.iter().map(|id| DsnpPrid::from(*id)).collect());
	page
}

/// Create test data for a single page
pub fn create_test_ids_and_page() -> (Vec<DsnpUserId>, GraphPage) {
	let ids: Vec<DsnpUserId> = vec![1u64, 2u64, 3u64].to_vec();
	let page = create_page(&ids);
	(ids, page)
}

/// Create a test instance of a Graph
pub fn create_test_graph() -> Graph {
	let num_pages = 5;
	let ids_per_page = 5;
	let mut curr_id = 0u64;
	let mut graph = Graph::new(ConnectionType::Follow(PrivacyType::Private));
	let mut pages = Vec::<GraphPage>::new();
	for _ in 0..num_pages {
		let ids: Vec<DsnpUserId> = (curr_id..(curr_id + ids_per_page)).collect();
		let page = create_page(&ids);
		pages.push(page);
		curr_id += ids_per_page;
	}

	for (i, p) in pages.iter().enumerate() {
		let _ = graph.create_page(&(i as PageId), Some(p.clone()));
	}

	graph
}

pub const INNER_TEST_DATA: [DsnpGraphEdge; 24] = [
	DsnpGraphEdge { user_id: 4464346407956074433, since: 8764139209126768069 },
	DsnpGraphEdge { user_id: 6668873909761685247, since: 7188698398086794482 },
	DsnpGraphEdge { user_id: 3983583835435595748, since: 829969197675906694 },
	DsnpGraphEdge { user_id: 5786399658613658850, since: 1167130351887327801 },
	DsnpGraphEdge { user_id: 2550476024131609410, since: 3207336660582066677 },
	DsnpGraphEdge { user_id: 8998781204841458437, since: 6168655822672170066 },
	DsnpGraphEdge { user_id: 2295352874227852087, since: 8440514722944450399 },
	DsnpGraphEdge { user_id: 2614565340217427162, since: 1493098497079203084 },
	DsnpGraphEdge { user_id: 4565430723166717193, since: 524506678053007723 },
	DsnpGraphEdge { user_id: 5906091589969275177, since: 6902573244786247664 },
	DsnpGraphEdge { user_id: 7159305214820893538, since: 1936283288692888565 },
	DsnpGraphEdge { user_id: 8396161706254593904, since: 4536230715384416065 },
	DsnpGraphEdge { user_id: 8854381008488607807, since: 5159191892139543717 },
	DsnpGraphEdge { user_id: 73771519320842737, since: 2000265679509608646 },
	DsnpGraphEdge { user_id: 5927922952678211908, since: 7047213894547814807 },
	DsnpGraphEdge { user_id: 7267061036641634127, since: 5580380300958088425 },
	DsnpGraphEdge { user_id: 8662377975562298354, since: 9159136102447625539 },
	DsnpGraphEdge { user_id: 1567949913908946319, since: 4616269828673275240 },
	DsnpGraphEdge { user_id: 7106429197891368988, since: 1323323443768786584 },
	DsnpGraphEdge { user_id: 8402348483076003273, since: 8296993699355902565 },
	DsnpGraphEdge { user_id: 5584173321377371204, since: 1019201472789084023 },
	DsnpGraphEdge { user_id: 2998808192952224961, since: 8286911785053584720 },
	DsnpGraphEdge { user_id: 2554776608916995203, since: 7585826393836986397 },
	DsnpGraphEdge { user_id: 4944236923077661927, since: 5383633821359802131 },
];

pub fn avro_inner_payload() -> Vec<u8> {
	SchemaHandler::write_inner_graph(&INNER_TEST_DATA.to_vec()).unwrap()
}

pub fn avro_compressed_inner_payload() -> Vec<u8> {
	DeflateCompression::compress(&avro_inner_payload()).unwrap()
}

pub fn avro_encrypted_inner_payload<E: EncryptionBehavior>(public_key: &PublicKey<E>) -> Vec<u8> {
	E::encrypt(&avro_compressed_inner_payload(), public_key).unwrap()
}

pub fn avro_public_payload() -> Vec<u8> {
	// encoded payload below matches INNER_TEST_DATA wrapped in a DsnpUserPublicGraphChunk
	let b64_payload = b"pgcBzgEx/jCCv5qI9dnF9HuKt5ehpIu9oPMB/tu5s5vfzIy5AeSzkPbhxrHDxwHI34PCjpnEyG6Mmuif0KXShBfE097BnvW1zaAB8oiR7eG3vbIghI2fqMqbjuVG6qenkePs34JZiqzL4ID1i+L5AaSxqMC18rubqwHujK/o49Hd2j++/ffo9trdouoBtJOqm+HU5shImNSa0YPLxrgpkpSO0LrD1dt+1sXbuu6CtscO0qTT8MvA0/ajAeCfzLnu8u7KvwHEjYKAqYr72sYB6s+E8YzZh9814L3XoMmllIXpAYKdopun8/bzff7A7c6rp4rh9QHKsuywn+yQmY8B4ofHusqzi4YCjOOaubTCr8I3iIWopJOhm8SkAa7mwYvh7N3MwwGendqh1ODk2ckB0vON46mUv/GaAeSP9/jzt/m28AGG1cW9wfDkm/4BnpatwvWIvcIr0JXk1u+8pJCAAbjgu5yH7Y2fxQGw+sfMpeqx3SSS18zMsNuRm+kByomanO3z66TmAYjRyo6Wg/z+mgHu3a/C3ZX3pByCs5anuIr0nVOg9e+RmpeDgeYBhvLL78fysfRGuoD9xr26osbSAc7T0Nal0rqdiQGmnPCu+pvBtpUBAA==";
	general_purpose::STANDARD.decode(b64_payload).unwrap()
}
