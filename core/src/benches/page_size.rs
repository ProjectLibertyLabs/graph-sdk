use dsnp_graph_config::{ConnectionType, ALL_CONNECTION_TYPES};

use crate::tests::helpers::*;
use rand::{distributions::Uniform, thread_rng, Rng};
use std::{collections::hash_map::HashMap, path::PathBuf};

pub fn benchmark_page_capacity(connection_type: ConnectionType) -> (usize, usize) {
	const MAX_PAGE_SIZE: usize = 1024;
	let mut builder = PageDataBuilder::new(connection_type).with_noisy_creation_time(true);
	let ids = Uniform::new(0x4000000000000000 as u64, 0x7fffffffffffffff as u64);
	let best_compression_ids = Uniform::new(0x7fffffffffffff00 as u64, 0x7fffffffffffffff as u64);
	let mut rng = thread_rng();
	let mut last_result: (usize, usize) = (0, 0);

	let mut i = 0;
	loop {
		let dist = match i == 0 {
			true => best_compression_ids,
			false => ids,
		};
		let connection_id = rng.sample(dist);
		let prid = rng.sample(dist).into();
		builder = builder.with_page(1, &[(connection_id, connection_id - 1)], &[prid], 0);
		let pages = builder.build_with_size();
		let (page_len, page) = pages.first().expect("page should exist");
		let page_size = page.content.len();

		if page_size >= MAX_PAGE_SIZE {
			// println!(
			// 	"{:?} page full. # connections = {:?}, bytes = {:?}",
			// 	connection_type, last_result.0, last_result.1
			// );
			break
		}

		last_result = (*page_len, page_size);
		i += 1;
	}

	last_result
}

#[test]
fn calculate_page_capacities() {
	let mut capacity_map: HashMap<ConnectionType, usize> = HashMap::new();

	for c in ALL_CONNECTION_TYPES {
		let mut result_vec: Vec<usize> = Vec::new();
		for _ in 0..1000 {
			result_vec.push(benchmark_page_capacity(c).0);
		}
		result_vec.sort();
		capacity_map.insert(c, *result_vec.first().unwrap());
	}

	let code = format!(
			"use dsnp_graph_config::{{ConnectionType, ConnectionType::Follow, ConnectionType::Friendship, PrivacyType::Public, PrivacyType::Private}};
use lazy_static::lazy_static;
use std::collections::hash_map::*;

lazy_static! {{
	pub static ref PAGE_CAPACITIY_MAP: HashMap<ConnectionType, usize> = {{
		let m = HashMap::from({:?});
		m
	}};
}}
",
			capacity_map.iter().collect::<Vec::<(&ConnectionType, &usize)>>()
		);

	let mut path = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
	path.push("src/graph/page_capacities.rs");

	let result = std::fs::write(path, code);
	if result.is_err() {
		println!("Error: {:?}", result);
	}

	assert!(true);
}
