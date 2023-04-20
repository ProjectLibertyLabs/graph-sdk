use crate::dsnp::{
	api_types::PageData,
	dsnp_types::{DsnpPrid, DsnpUserId},
	schema::SchemaHandler,
};
use anyhow::Result;
use std::collections::HashMap;

/// A trait that defines all the functionality that a pri manager should implement.
pub trait PriProvider {
	/// imports pri for a user and replaces the older ones if exists
	fn import_pri(&mut self, dsnp_user_id: DsnpUserId, pages: &[PageData]) -> Result<()>;

	/// checks if a pri exist for a specific user
	fn contains(&self, dsnp_user_id: DsnpUserId, prid: DsnpPrid) -> bool;
}

#[derive(Debug, Eq, PartialEq)]
pub struct PriManager {
	/// keys are stored sorted by index
	dsnp_user_to_pris: HashMap<DsnpUserId, Vec<DsnpPrid>>,
}

impl PriProvider for PriManager {
	fn import_pri(&mut self, dsnp_user_id: DsnpUserId, pages: &[PageData]) -> Result<()> {
		let mut prids = vec![];
		for p in pages {
			let chunk = SchemaHandler::read_private_graph_chunk(&p.content[..])?;
			prids.extend_from_slice(&chunk.prids);
		}
		self.dsnp_user_to_pris.insert(dsnp_user_id, prids);
		Ok(())
	}

	fn contains(&self, dsnp_user_id: DsnpUserId, prid: DsnpPrid) -> bool {
		self.dsnp_user_to_pris.get(&dsnp_user_id).unwrap_or(&Vec::new()).contains(&prid)
	}
}

impl PriManager {
	pub fn new() -> Self {
		Self { dsnp_user_to_pris: HashMap::new() }
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use crate::tests::helpers::PageDataBuilder;
	use dsnp_graph_config::{ConnectionType::Friendship, PrivacyType};

	#[test]
	fn pri_manager_import_should_store_prids_as_expected() {
		// arrange
		let mut manager = PriManager::new();
		let prid_1 = DsnpPrid::new(&[1u8, 2, 3, 4, 5, 6, 7, 8]);
		let prid_2 = DsnpPrid::new(&[10u8, 20, 3, 4, 5, 6, 7, 8]);
		let non_existing_prid = DsnpPrid::new(&[2u8, 20, 3, 4, 5, 6, 7, 8]);
		let pages = PageDataBuilder::new(Friendship(PrivacyType::Private))
			.with_page(1, &vec![1], &vec![prid_1.clone()])
			.with_page(2, &vec![2], &vec![prid_2.clone()])
			.build();
		let dsnp_user_id = 23;
		let non_existing_user_id = 10;

		// act
		let res = manager.import_pri(dsnp_user_id, &pages);

		// assert
		assert!(res.is_ok());
		assert_eq!(
			manager.dsnp_user_to_pris.get(&dsnp_user_id),
			Some(&vec![prid_1.clone(), prid_2.clone()])
		);
		assert_eq!(manager.dsnp_user_to_pris.get(&non_existing_user_id), None);
		assert!(manager.contains(dsnp_user_id, prid_1));
		assert!(manager.contains(dsnp_user_id, prid_2));
		assert!(!manager.contains(dsnp_user_id, non_existing_prid));
	}

	#[test]
	fn pri_manager_import_should_replace_previous_prids() {
		// arrange
		let mut manager = PriManager::new();
		let old_prid = DsnpPrid::new(&[1u8, 2, 3, 4, 5, 6, 7, 8]);
		let pages = PageDataBuilder::new(Friendship(PrivacyType::Private))
			.with_page(1, &vec![1], &vec![old_prid])
			.build();
		let dsnp_user_id = 23;
		manager.import_pri(dsnp_user_id, &pages).expect("should work");
		let new_prid = DsnpPrid::new(&[10u8, 20, 30, 40, 50, 60, 70, 80]);
		let new_pages = PageDataBuilder::new(Friendship(PrivacyType::Private))
			.with_page(1, &vec![1], &vec![new_prid.clone()])
			.build();

		// act
		let res = manager.import_pri(dsnp_user_id, &new_pages);

		// assert
		assert!(res.is_ok());
		assert_eq!(manager.dsnp_user_to_pris.get(&dsnp_user_id), Some(&vec![new_prid]));
	}
}
