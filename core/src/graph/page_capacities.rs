use dsnp_graph_config::{
	ConnectionType,
	ConnectionType::{Follow, Friendship},
	PrivacyType::{Private, Public},
};
use lazy_static::lazy_static;
use std::collections::hash_map::*;

lazy_static! {
	pub static ref PAGE_CAPACITIY_MAP: HashMap<ConnectionType, usize> = {
		let m = HashMap::from([
			(Follow(Private), 88),
			(Friendship(Public), 93),
			(Follow(Public), 93),
			(Friendship(Private), 49),
		]);
		m
	};
}
