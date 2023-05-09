use dsnp_graph_config::{ConnectionType, PrivacyType};
use lazy_static::lazy_static;
use std::collections::hash_map::*;

lazy_static! {
	pub static ref PAGE_CAPACITIY_MAP: HashMap<ConnectionType, usize> = {
		let m = HashMap::from([(Follow(Public), 93), (Follow(Private), 88), (Friendship(Private), 49), (Friendship(Public), 93)]);
		m
	};
}
