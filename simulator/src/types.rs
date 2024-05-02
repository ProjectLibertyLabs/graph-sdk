use dsnp_graph_config::{DsnpUserId, Environment, SchemaId};
use dsnp_graph_core::api::api_types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Serialize, Deserialize, PartialEq, Debug, Clone)]
pub enum Status {
	Init,
	InitialUsersCreated,
	InitialKeysCreated,
	PrivateFollowsCreated,
	PrivateFriendshipsCreated,
	End,
}

impl Default for Status {
	fn default() -> Self {
		Status::Init
	}
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default)]
pub struct GlobalState {
	pub current_status: Status,
	pub users: Vec<DsnpUserId>,
	pub social_graph: HashMap<(DsnpUserId, SchemaId), Vec<DsnpUserId>>,
	pub on_chain_keys: HashMap<(DsnpUserId, SchemaId), DsnpKeys>,
	pub on_chain_graph: HashMap<(DsnpUserId, SchemaId), Vec<PageData>>,
	pub wallet_keys: HashMap<DsnpUserId, Vec<GraphKeyPair>>,
}

#[derive(Serialize, Deserialize, PartialEq, Debug, Default, Clone)]
pub struct TempData {
	pub user_id: DsnpUserId,
	pub schema_id: SchemaId,
	pub connections: Vec<DsnpUserId>,
	pub pages: Vec<PageData>,
}

impl GlobalState {
	pub fn get_all_data_for_user(
		&self,
		env: Environment,
		user_id: DsnpUserId,
		schema_id: SchemaId,
	) -> (DsnpKeys, Vec<GraphKeyPair>, Vec<PageData>, Vec<DsnpUserId>) {
		let public_key_schema_id = env.get_config().graph_public_key_schema_id;

		// get uploaded keys from chain
		let user_dsnp_keys =
			self.on_chain_keys.get(&(user_id, public_key_schema_id)).expect("Should exist");
		// get key pairs from wallet
		let user_key_pairs = self.wallet_keys.get(&user_id).expect("Should exist");
		// get uploaded pages
		let temp_page_data = vec![];
		let user_pages = self.on_chain_graph.get(&(user_id, schema_id)).unwrap_or(&temp_page_data);
		// get expected social graph
		let temp_social_graph = vec![];
		let social_graph =
			self.social_graph.get(&(user_id, schema_id)).unwrap_or(&temp_social_graph);
		(user_dsnp_keys.clone(), user_key_pairs.clone(), user_pages.clone(), social_graph.clone())
	}

	pub fn apply_updates_for_user(
		&mut self,
		env: Environment,
		user_id: DsnpUserId,
		graph_schema_id: SchemaId,
		updates: &[Update],
		adds: &[DsnpUserId],
		removes: &[DsnpUserId],
		keypair: Option<&GraphKeyPair>, // since it doesn't make sense to add multiple keys to the user at any time we are just using one keypair
	) {
		let public_key_schema_id = env.get_config().graph_public_key_schema_id;
		let social_changes = self.social_graph.get_mut(&(user_id, graph_schema_id)).unwrap();
		social_changes.retain(|item| !removes.contains(item));
		social_changes.extend(adds);

		let on_chain_graph = self.on_chain_graph.get_mut(&(user_id, graph_schema_id)).unwrap();

		let on_chain_keys = self.on_chain_keys.get_mut(&(user_id, public_key_schema_id)).unwrap();
		for u in updates {
			match u {
				Update::DeletePage { page_id, prev_hash, owner_dsnp_user_id, schema_id } => {
					assert_eq!(user_id, *owner_dsnp_user_id);
					assert_eq!(graph_schema_id, *schema_id);

					on_chain_graph
						.retain(|p| !(p.page_id == *page_id && p.content_hash == *prev_hash));
				},
				Update::PersistPage {
					page_id,
					prev_hash,
					payload,
					owner_dsnp_user_id,
					schema_id,
				} => {
					assert_eq!(user_id, *owner_dsnp_user_id);
					assert_eq!(graph_schema_id, *schema_id);

					on_chain_graph
						.retain(|p| !(p.page_id == *page_id && p.content_hash == *prev_hash));
					on_chain_graph.push(PageData {
						content: payload.clone(),
						content_hash: prev_hash + 1,
						page_id: *page_id,
					});
				},
				Update::AddKey { owner_dsnp_user_id, prev_hash, payload } => {
					assert_eq!(user_id, *owner_dsnp_user_id);
					assert_eq!(&on_chain_keys.keys_hash, prev_hash);

					on_chain_keys.keys_hash += 1;
					on_chain_keys.keys.push(KeyData {
						content: payload.clone(),
						index: on_chain_keys.keys.len() as u16,
					});

					self.wallet_keys.entry(user_id).or_default().push(keypair.unwrap().clone());
				},
			}
		}
	}

	pub fn get_all_users_in_graph_for(&self, schema_id: SchemaId) -> Vec<DsnpUserId> {
		self.on_chain_graph
			.iter()
			.filter_map(|(&(dsnp_user, schema), _)| match schema_id == schema {
				true => Some(dsnp_user),
				false => None,
			})
			.collect()
	}

	pub fn prepare_all_import_bundles(
		&self,
		env: Environment,
		user_id: DsnpUserId,
		schema_id: SchemaId,
		is_friendship: bool,
	) -> (Vec<ImportBundle>, Vec<DsnpUserId>) {
		let (user_dsnp_keys, user_key_pairs, pages, social_graph) =
			self.get_all_data_for_user(env.clone(), user_id, schema_id);
		// prepare all bundles
		let mut all_bundles = vec![ImportBundle {
			schema_id,
			dsnp_user_id: user_id,
			pages,
			dsnp_keys: Some(user_dsnp_keys.clone()),
			key_pairs: user_key_pairs.clone(),
		}];
		if is_friendship {
			let friend_bundles: Vec<_> = social_graph
				.iter()
				.map(|c| {
					let (friend_dsnp_keys, _, friend_pages, _) =
						self.get_all_data_for_user(env.clone(), *c, schema_id);
					ImportBundle {
						schema_id,
						dsnp_user_id: *c,
						pages: friend_pages,
						dsnp_keys: Some(friend_dsnp_keys.clone()),
						key_pairs: vec![],
					}
				})
				.collect();
			all_bundles.extend(friend_bundles.into_iter());
		}
		(all_bundles, social_graph)
	}
}
