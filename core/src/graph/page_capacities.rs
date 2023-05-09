use dsnp_graph_config::{ConnectionType, ConnectionType::Follow, ConnectionType::Friendship, PrivacyType::Public, PrivacyType::Private};
use lazy_static::lazy_static;
use std::collections::hash_map::*;

lazy_static! {
	pub static ref PAGE_CAPACITIY_MAP: HashMap<ConnectionType, usize> = {
		let m = HashMap::from([(Friendship(Public), 92), (Follow(Private), 88), (Friendship(Private), 49), (Follow(Public), 93)]);
		m
	};
}
