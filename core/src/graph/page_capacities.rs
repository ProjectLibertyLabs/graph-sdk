use dsnp_graph_config::{ConnectionType, PrivacyType};
use lazy_static::lazy_static;
use std::collections::hash_map::*;

lazy_static! {
	pub static ref PAGE_CAPACITIY_MAP: HashMap<ConnectionType, usize> = {
		let m = HashMap::from([
			(ConnectionType::Friendship(PrivacyType::Public), 94),
			(ConnectionType::Follow(PrivacyType::Public), 93),
			(ConnectionType::Follow(PrivacyType::Private), 88),
			(ConnectionType::Friendship(PrivacyType::Private), 50),
		]);
		m
	};
}
