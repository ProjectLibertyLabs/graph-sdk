use crate::dsnp::{
	dsnp_types::{DsnpGraphEdge, DsnpPrid, DsnpUserId},
	graph_page::GraphPage,
};
use std::time::{SystemTime, UNIX_EPOCH};

pub fn create_graph_edge(id: &DsnpUserId) -> DsnpGraphEdge {
	DsnpGraphEdge {
		user_id: *id,
		since: SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs(),
	}
}

impl From<DsnpUserId> for DsnpPrid {
	fn from(id: DsnpUserId) -> Self {
		Self::from(id.to_le_bytes().to_vec())
	}
}

pub fn create_page(ids: &[DsnpUserId]) -> GraphPage {
	let mut page = GraphPage::new(crate::dsnp::api_types::PrivacyType::Private, 0);
	page.set_connections(ids.iter().map(create_graph_edge).collect());
	page.set_prids(ids.iter().map(|id| DsnpPrid::from(*id)).collect());
	page
}
