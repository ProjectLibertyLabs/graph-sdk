use crate::dsnp::dsnp_types::{DsnpInnerGraph, DsnpPrid};

#[derive(Debug, Clone, PartialEq)]
pub struct PrivateGraphChunk {
	/// User-Assigned Key Identifier
	pub key_id: u64,

	/// User-Assigned Key Identifier
	pub prids: Vec<DsnpPrid>,

	/// Unix epoch seconds when PRIds for this chunk were last refreshed, rounded to the nearest 1000
	pub last_updated: u64,

	/// connections
	pub inner_graph: DsnpInnerGraph,
}
