use crate::{
	dsnp::{
		api_types::*,
		dsnp_configs::{DsnpVersionConfig, KeyPairType},
		dsnp_types::*,
	},
	graph::{
		graph::{Graph, PageFullnessMode},
		key_manager::{UserKeyManager, UserKeyProvider},
		page::GraphPage,
		page_capacities::PAGE_CAPACITY_MAP,
		shared_state_manager::{PublicKeyProvider, SharedStateManager},
	},
	util::{
		builders::{GraphPageBuilder, KeyDataBuilder},
		time::time_in_ksecs,
	},
};
use base64::{engine::general_purpose, Engine as _};
use ctor::ctor;
use dryoc::keypair::StackKeyPair;
use dsnp_graph_config::{DsnpVersion, Environment, GraphKeyType};
use std::sync::{Arc, RwLock};

#[ctor]
fn test_harness_init() {
	const IS_TEST: bool = true; // set to false to see log output in tests
	let _ = env_logger::builder().is_test(IS_TEST).try_init();
}

pub fn create_graph_edge(id: &DsnpUserId) -> DsnpGraphEdge {
	DsnpGraphEdge { user_id: *id, since: time_in_ksecs() }
}

impl From<DsnpUserId> for DsnpPrid {
	fn from(id: DsnpUserId) -> Self {
		Self::from(id.to_le_bytes().to_vec())
	}
}

/// Create test data for a single page
pub fn create_test_ids_and_page() -> (Vec<(DsnpUserId, u64)>, GraphPage) {
	let ids: Vec<(DsnpUserId, u64)> = vec![(1u64, 0), (2u64, 0), (3u64, 0)].to_vec();
	let pages = GraphPageBuilder::new(ConnectionType::Follow(PrivacyType::Private))
		.with_page(1, &ids, &vec![], 0)
		.build();
	let page = pages.first().expect("page should exist").clone();
	(ids, page)
}

/// Get config environment and schema ID for a connection type
pub fn get_env_and_config() -> (Environment, DsnpVersionConfig) {
	let env = Environment::Mainnet;
	let config = DsnpVersionConfig::new(DsnpVersion::Version1_0);
	(env, config)
}

/// Create an empty test instance of a Graph
pub fn create_empty_test_graph(
	user_id: Option<DsnpUserId>,
	connection_arg: Option<ConnectionType>,
) -> (Graph, Arc<RwLock<UserKeyManager>>, Arc<RwLock<SharedStateManager>>) {
	let connection_type = match connection_arg {
		Some(c) => c,
		None => ConnectionType::Follow(PrivacyType::Private),
	};
	let user_id = user_id.unwrap_or(3u64);

	let (env, _) = get_env_and_config();
	let key = ResolvedKeyPair { key_id: 1, key_pair: KeyPairType::Version1_0(StackKeyPair::gen()) };
	let shared_state = Arc::new(RwLock::new(SharedStateManager::new()));
	let user_key_mgr = Arc::new(RwLock::new(UserKeyManager::new(user_id, shared_state.clone())));

	shared_state
		.write()
		.unwrap()
		.import_keys_test(
			user_id,
			&vec![DsnpPublicKey {
				key_id: Some(key.key_id),
				key: key.key_pair.get_public_key_raw(),
			}],
			0,
		)
		.expect("should insert keys");
	user_key_mgr
		.write()
		.unwrap()
		.import_key_pairs(vec![GraphKeyPair {
			key_type: GraphKeyType::X25519,
			public_key: key.key_pair.get_public_key_raw(),
			secret_key: key.key_pair.get_secret_key_raw(),
		}])
		.expect("should import user keys");
	let graph = Graph::new(
		env.clone(),
		user_id,
		env.get_config()
			.get_schema_id_from_connection_type(connection_type)
			.expect("should get schema id"),
		user_key_mgr.clone(),
	);

	(graph, user_key_mgr, shared_state)
}

/// Create a page that is trivially full
pub fn create_trivially_full_page(
	connection_type: ConnectionType,
	page_id: PageId,
	start_conn_id: u64,
) -> GraphPage {
	let mut page = GraphPage::new(connection_type.privacy_type(), page_id);
	// let builder = GraphPageBuilder::new(connection_type);
	let max_connections_per_page = *PAGE_CAPACITY_MAP.get(&connection_type).unwrap_or_else(|| {
		let mut capacities: Vec<&usize> = PAGE_CAPACITY_MAP.values().collect();
		capacities.sort();
		capacities.first().unwrap() // default: return smallest capacity value
	});
	let mut curr_id = start_conn_id;
	while page.connections().len() < max_connections_per_page {
		page.add_connection(&curr_id).expect("unable to add connection");
		curr_id += 1;
	}

	page
}

/// Create a page that is aggressively full
pub fn create_aggressively_full_page(
	graph: &mut Graph,
	start_conn_id: u64,
	dsnp_version_config: &DsnpVersionConfig,
	shared_state: &Arc<RwLock<SharedStateManager>>,
) -> PageId {
	let connection_type = graph.get_connection_type();
	let page_id = graph.get_next_available_page_id().unwrap();
	let mut page = GraphPage::new(connection_type.privacy_type(), page_id);
	let mut connection_id = start_conn_id;
	let encryption_key = graph
		.get_user_key_mgr()
		.read()
		.unwrap()
		.get_resolved_active_key(graph.get_dsnp_user_id());

	loop {
		if connection_type == ConnectionType::Friendship(PrivacyType::Private) {
			add_public_key_for_dsnp_id(connection_id, shared_state);
		}

		if graph
			.try_add_connection_to_page(
				&mut page,
				&connection_id,
				PageFullnessMode::Aggressive,
				dsnp_version_config,
				&encryption_key,
			)
			.is_err()
		{
			break
		}

		connection_id += 1;
	}

	graph.create_page(&page_id, Some(page)).expect("failed to add page to graph");
	page_id
}

/// Create a test instance of a Graph
pub fn create_test_graph(connection_arg: Option<ConnectionType>) -> Graph {
	let connection_type = match connection_arg {
		Some(c) => c,
		None => ConnectionType::Follow(PrivacyType::Private),
	};
	let mut page_builder = GraphPageBuilder::new(connection_type);
	let num_pages = 5;
	let ids_per_page = 5;
	let mut curr_id = 0u64;
	for i in 0..num_pages {
		let ids: Vec<(DsnpUserId, u64)> =
			(curr_id..(curr_id + ids_per_page)).map(|u| (u, 0)).collect();
		let prids = match connection_type {
			ConnectionType::Friendship(PrivacyType::Private) =>
				ids.iter().cloned().map(|(id, _)| DsnpPrid::from(id)).collect(),
			_ => Vec::<DsnpPrid>::new(),
		};
		page_builder = page_builder.with_page(i, &ids, &prids, 0);
		curr_id += ids_per_page;
	}

	let (mut graph, ..) = create_empty_test_graph(None, Some(connection_type));
	for p in page_builder.build() {
		let _ = graph.create_page(&p.page_id(), Some(p));
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

pub fn avro_public_payload() -> Vec<u8> {
	// encoded payload below matches INNER_TEST_DATA wrapped in a DsnpUserPublicGraphChunk
	let b64_payload = b"pgcBzgEx/jCCv5qI9dnF9HuKt5ehpIu9oPMB/tu5s5vfzIy5AeSzkPbhxrHDxwHI34PCjpnEyG6Mmuif0KXShBfE097BnvW1zaAB8oiR7eG3vbIghI2fqMqbjuVG6qenkePs34JZiqzL4ID1i+L5AaSxqMC18rubqwHujK/o49Hd2j++/ffo9trdouoBtJOqm+HU5shImNSa0YPLxrgpkpSO0LrD1dt+1sXbuu6CtscO0qTT8MvA0/ajAeCfzLnu8u7KvwHEjYKAqYr72sYB6s+E8YzZh9814L3XoMmllIXpAYKdopun8/bzff7A7c6rp4rh9QHKsuywn+yQmY8B4ofHusqzi4YCjOOaubTCr8I3iIWopJOhm8SkAa7mwYvh7N3MwwGendqh1ODk2ckB0vON46mUv/GaAeSP9/jzt/m28AGG1cW9wfDkm/4BnpatwvWIvcIr0JXk1u+8pJCAAbjgu5yH7Y2fxQGw+sfMpeqx3SSS18zMsNuRm+kByomanO3z66TmAYjRyo6Wg/z+mgHu3a/C3ZX3pByCs5anuIr0nVOg9e+RmpeDgeYBhvLL78fysfRGuoD9xr26osbSAc7T0Nal0rqdiQGmnPCu+pvBtpUBAA==";
	general_purpose::STANDARD.decode(b64_payload).unwrap()
}

pub fn add_public_key_for_dsnp_id(
	dsnp_user_id: DsnpUserId,
	shared_state: &Arc<RwLock<SharedStateManager>>,
) {
	let dsnp_keys = DsnpKeys {
		dsnp_user_id,
		keys_hash: 0,
		keys: KeyDataBuilder::new().with_generated_key().build(),
	};

	shared_state
		.write()
		.unwrap()
		.import_dsnp_keys(&dsnp_keys)
		.expect("failed to import public key");
}
