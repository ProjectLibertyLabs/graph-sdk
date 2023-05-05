#[cfg_attr(all(test, not(feature = "calculate-page-capacity")), allow(dead_code))]
use crate::{
	dsnp::{
		api_types::{GraphKeyPair, PageData, ResolvedKeyPair},
		dsnp_configs::SecretKeyType,
		dsnp_types::{DsnpPrid, DsnpUserId},
	},
	graph::{
		key_manager::{ConnectionVerifier, UserKeyManagerBase, UserKeyProvider},
		shared_state_manager::PriProvider,
	},
};
use anyhow::{Error, Result};
use std::{collections::HashMap, fmt::Debug};

#[derive(Debug)]
pub struct MockUserKeyManager {
	verifications: HashMap<DsnpUserId, Option<bool>>,
}

#[cfg_attr(feature = "calculate-page-capacity", allow(dead_code))]
impl MockUserKeyManager {
	pub fn new() -> Self {
		Self { verifications: HashMap::<DsnpUserId, Option<bool>>::default() }
	}

	pub fn register_verifications(&mut self, pairs: &[(DsnpUserId, Option<bool>)]) {
		pairs.iter().for_each(|(user, verified)| {
			self.verifications.insert(*user, *verified);
		})
	}
}

impl ConnectionVerifier for MockUserKeyManager {
	fn verify_connection(&self, from: DsnpUserId) -> Result<bool> {
		match self.verifications.get(&from) {
			Some(Some(verified)) => Ok(*verified),
			Some(None) => Err(Error::msg("Generic error in verifier!")),
			None => Err(Error::msg("Non registered use!r")),
		}
	}
}

impl UserKeyProvider for MockUserKeyManager {
	fn import_key_pairs(&mut self, _pairs: Vec<GraphKeyPair>) -> Result<()> {
		Ok(())
	}

	fn get_resolved_key(&self, _key_id: u64) -> Option<ResolvedKeyPair> {
		None
	}

	fn get_all_resolved_keys(&self) -> Vec<ResolvedKeyPair> {
		vec![]
	}

	fn get_resolved_active_key(&self, _dsnp_user_id: DsnpUserId) -> Option<ResolvedKeyPair> {
		None
	}
}

impl PriProvider for MockUserKeyManager {
	fn import_pri(&mut self, _dsnp_user_id: DsnpUserId, _pages: &[PageData]) -> Result<()> {
		Ok(())
	}

	fn contains(&self, _dsnp_user_id: DsnpUserId, _prid: DsnpPrid) -> bool {
		true
	}

	fn calculate_prid(
		&self,
		_from: DsnpUserId,
		_to: DsnpUserId,
		_from_secret: SecretKeyType,
	) -> Result<DsnpPrid> {
		Ok(DsnpPrid::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7]))
	}
}

impl UserKeyManagerBase for MockUserKeyManager {}
