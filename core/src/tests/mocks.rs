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
use dsnp_graph_config::errors::{DsnpGraphError, DsnpGraphResult};
use std::{collections::HashMap, fmt::Debug};

#[derive(Debug)]
pub struct MockUserKeyManager {
	verifications: HashMap<DsnpUserId, Option<bool>>,
	key_pairs: HashMap<DsnpUserId, Vec<ResolvedKeyPair>>,
}

impl MockUserKeyManager {
	pub fn new() -> Self {
		Self {
			verifications: HashMap::<DsnpUserId, Option<bool>>::default(),
			key_pairs: HashMap::new(),
		}
	}

	pub fn register_verifications(&mut self, pairs: &[(DsnpUserId, Option<bool>)]) {
		pairs.iter().for_each(|(user, verified)| {
			self.verifications.insert(*user, *verified);
		})
	}

	pub fn register_key(&mut self, dsnp_user_id: DsnpUserId, pair: &ResolvedKeyPair) {
		self.key_pairs.entry(dsnp_user_id).or_default().push(pair.clone());
	}
}

impl ConnectionVerifier for MockUserKeyManager {
	fn verify_connection(&self, from: DsnpUserId) -> DsnpGraphResult<bool> {
		match self.verifications.get(&from) {
			Some(Some(verified)) => Ok(*verified),
			Some(None) =>
				Err(DsnpGraphError::UnknownError("generic verification error!".to_string())),
			None => Err(DsnpGraphError::UnknownError("Non registered user!".to_string())),
		}
	}
}

impl UserKeyProvider for MockUserKeyManager {
	fn import_key_pairs(&mut self, _pairs: Vec<GraphKeyPair>) -> DsnpGraphResult<()> {
		Ok(())
	}

	fn get_resolved_key(&self, _key_id: u64) -> Option<ResolvedKeyPair> {
		None
	}

	fn get_all_resolved_keys(&self) -> Vec<ResolvedKeyPair> {
		vec![]
	}

	fn get_resolved_active_key(&self, dsnp_user_id: DsnpUserId) -> Option<ResolvedKeyPair> {
		match self.key_pairs.get(&dsnp_user_id) {
			None => None,
			Some(keys) => keys.last().cloned(),
		}
	}
}

impl PriProvider for MockUserKeyManager {
	fn import_pri(
		&mut self,
		_dsnp_user_id: DsnpUserId,
		_pages: &[PageData],
	) -> DsnpGraphResult<()> {
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
	) -> DsnpGraphResult<DsnpPrid> {
		Ok(DsnpPrid::from(vec![0u8, 1, 2, 3, 4, 5, 6, 7]))
	}
}

impl UserKeyManagerBase for MockUserKeyManager {}
