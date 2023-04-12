use crate::dsnp::dsnp_types::{DsnpInnerGraph, DsnpPrid};

#[derive(Debug, Clone, PartialEq)]
pub struct PrivateGraphChunk {
	/// User-Assigned Key Identifier
	pub key_id: u64,

	/// User-Assigned Key Identifier
	pub prids: Vec<DsnpPrid>,

	/// connections
	pub inner_graph: DsnpInnerGraph,
}
