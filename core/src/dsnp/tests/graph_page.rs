use crate::{
	dsnp::{
		api_types::*,
		dsnp_types::*,
		graph_page::{Graph, GraphPage},
		tests::helpers::*,
	},
	iter_graph_connections,
};

/// Create test data for a single page
fn create_test_ids_and_page() -> (Vec<DsnpUserId>, GraphPage) {
	let ids: Vec<DsnpUserId> = vec![1u64, 2u64, 3u64].to_vec();
	let page = create_page(&ids);
	(ids, page)
}

/// Create a test instance of a Graph
fn create_test_graph() -> Graph {
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

mod page_tests {
	use super::*;
	#[allow(unused_imports)]
	use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};

	#[test]
	fn new_page() {
		let page = GraphPage::new(PrivacyType::Private, 0);

		assert_eq!(page.is_empty(), true, "Page should be empty");
	}

	#[test]
	fn graph_page_getters_setters() {
		let mut page = GraphPage::new(PrivacyType::Private, 0);
		let prids: Vec<DsnpPrid> = vec![1, 2, 3, 4].iter().map(|id| DsnpPrid::from(*id)).collect();
		let connections: Vec<DsnpGraphEdge> =
			vec![5, 6, 7, 8].iter().map(create_graph_edge).collect();

		page.set_prids(prids.clone());
		page.set_connections(connections.clone());
		assert_eq!(&prids, page.prids());
		assert_eq!(&connections, page.connections());
	}

	#[test]
	fn page_contains_finds_item() {
		let (ids, page) = create_test_ids_and_page();
		for id in ids {
			assert_eq!(page.contains(&id as &DsnpUserId), true);
		}
	}

	#[test]
	fn page_contains_does_not_find_missing_items() {
		let (_, page) = create_test_ids_and_page();
		assert_eq!(page.contains(&(4 as DsnpUserId)), false);
	}

	#[test]
	fn is_empty_on_nonempty_page_returns_false() {
		let (_, page) = create_test_ids_and_page();
		assert_eq!(page.is_empty(), false);
	}

	#[test]
	fn add_duplicate_connection_fails() {
		let (_, mut page) = create_test_ids_and_page();
		assert_eq!(page.add_connection(&1u64).is_err(), true);
	}

	#[test]
	fn add_connection_succeeds() {
		let id: DsnpUserId = 1;
		let mut page = GraphPage::new(PrivacyType::Private, 0);

		assert_eq!(page.add_connection(&id).is_ok(), true);
		assert_eq!(page.contains(&id), true);
	}

	#[test]
	fn remove_connection_not_found_fails() {
		let (_, mut page) = create_test_ids_and_page();

		assert_eq!(page.remove_connection(&4u64).is_err(), true);
	}

	#[test]
	fn remove_connection_succeeds() {
		let (_, mut page) = create_test_ids_and_page();
		let id_to_remove = 1u64;

		assert_eq!(page.remove_connection(&id_to_remove).is_ok(), true);
		assert_eq!(page.contains(&id_to_remove), false);
	}
}

mod graph_tests {
	use super::*;
	#[allow(unused_imports)]
	use pretty_assertions::{assert_eq, assert_ne, assert_str_eq};
	use std::collections::HashMap;

	#[test]
	fn new_graph_is_empty() {
		let graph = Graph::new(ConnectionType::Follow(PrivacyType::Private));
		assert_eq!(graph.pages().is_empty(), true);
	}

	#[test]
	fn page_setter_sets_pages() {
		let mut pages = HashMap::<PageId, GraphPage>::new();
		for i in 0..=1 {
			let (_, p) = create_test_ids_and_page();
			pages.insert(i, p);
		}
		let mut graph = Graph::new(ConnectionType::Follow(PrivacyType::Private));
		graph.set_pages(pages.clone());
		assert_eq!(pages.len(), graph.pages().len());
		for i in 0..pages.len() as u16 {
			assert_eq!(pages.get(&i), graph.pages().get(&i));
		}
	}

	#[test]
	fn create_page_with_existing_pageid_fails() {
		let mut graph = create_test_graph();

		assert_eq!(graph.create_page(&0, None).is_err(), true);
	}

	#[test]
	fn create_page_succeeds() {
		let (_, page) = create_test_ids_and_page();
		let mut graph = Graph::new(ConnectionType::Follow(PrivacyType::Private));

		assert_eq!(graph.create_page(&0, Some(page.clone())).is_ok(), true);
		assert_eq!(page, *graph.get_page(&0).unwrap());
	}

	#[test]
	fn find_connection_returns_none_for_nonexistent_connection() {
		let graph = create_test_graph();

		assert_eq!(graph.find_connection(&99), None);
	}

	#[test]
	fn find_connections_returns_pageid_of_existing_connection() {
		let graph = create_test_graph();

		assert_eq!(graph.find_connection(&1), Some(0));
	}

	#[test]
	fn add_connection_duplicate_connection_errors() {
		let mut graph = create_test_graph();

		assert_eq!(graph.add_connection_to_page(&4, &0).is_err(), true);
	}

	#[test]
	fn add_connection_to_nonexistent_page_adds_new_page() {
		let mut graph = create_test_graph();
		let page_to_add: PageId = 99;

		assert_eq!(graph.pages().contains_key(&page_to_add), false);
		let _ = graph.add_connection_to_page(&page_to_add, &12345);
		assert_eq!(graph.pages().contains_key(&page_to_add), true);
	}

	#[test]
	fn add_connection_succeeds() {
		let mut graph = create_test_graph();

		let _ = graph.add_connection_to_page(&4, &99);
		assert_eq!(graph.find_connection(&99), Some(4));
	}

	#[test]
	fn remove_connection_returns_none_for_not_found() {
		let mut graph = create_test_graph();

		let result = graph.remove_connection(&99);
		assert_eq!(result.unwrap().is_none(), true);
	}

	#[test]
	fn remove_connection_returns_pageid_of_removed_connection() {
		let mut graph = create_test_graph();

		let result = graph.remove_connection(&5);
		assert_eq!(result.unwrap(), Some(1));
	}

	#[test]
	fn graph_iterator_should_iterate_over_all_connections() {
		let graph = create_test_graph();
		let mut test_connections: Vec<DsnpUserId> = (0..25).map(|i| i as DsnpUserId).collect();
		test_connections.sort();

		let mut graph_connections: Vec<DsnpUserId> =
			iter_graph_connections!(graph).map(|edge| edge.user_id).collect();
		graph_connections.sort();
		assert_eq!(test_connections, graph_connections);
	}
}
