use dsnp_graph_config::{ConnectionType, PrivacyType};
use lazy_static::lazy_static;
use std::collections::hash_map::*;

lazy_static! {
	pub static ref PAGE_CAPACITIY_MAP: HashMap<ConnectionType, usize> = {
		let m = HashMap::from([
			(ConnectionType::Follow(PrivacyType::Public), 107),
			(ConnectionType::Friendship(PrivacyType::Public), 107),
			(ConnectionType::Follow(PrivacyType::Private), 101),
			(ConnectionType::Friendship(PrivacyType::Private), 53),
		]);
		m
	};
}
