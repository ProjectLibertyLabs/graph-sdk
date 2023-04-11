#![allow(dead_code)] // todo: remove after usage
use crate::{
	dsnp::{
		api_types::{ConnectionType, DsnpKeys, ExportBundle, PublicKey},
		dsnp_types::DsnpUserId,
		encryption::EncryptionBehavior,
	},
	graph::{
		updates::UpdateEvent::{Add, Remove},
		user::UserGraph,
	},
};
use anyhow::{Error, Result};
use std::{cmp::Ordering, collections::HashMap};

#[derive(Clone, PartialEq, Ord, Eq, PartialOrd, Debug)]
pub enum UpdateEvent {
	Add { dsnp_user_id: DsnpUserId, connection_type: ConnectionType },
	Remove { dsnp_user_id: DsnpUserId, connection_type: ConnectionType },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct UpdateTracker {
	updates: HashMap<ConnectionType, Vec<UpdateEvent>>,
}

impl UpdateTracker {
	pub fn new() -> Self {
		Self { updates: HashMap::new() }
	}

	pub fn register_update(&mut self, event: &UpdateEvent) -> Result<()> {
		if self.contains(event) {
			return Err(Error::msg("event exists"))
		}

		match self.contains_complement(event) {
			// removing the complement to cancel out a prior update
			true => self.remove(&event.get_complement()),
			// adding new update
			false => self.add(event),
		}

		Ok(())
	}

	pub fn register_updates(&mut self, events: &[UpdateEvent]) -> Result<()> {
		if events.iter().any(|e| self.contains(e)) {
			return Err(Error::msg("one or more events exist"))
		}

		for e in events {
			self.register_update(e)?;
		}

		Ok(())
	}

	pub fn has_updates(&self) -> bool {
		self.updates.iter().any(|(_, v)| !v.is_empty())
	}

	pub fn get_updates_for_connection_type(
		&self,
		connection_type: ConnectionType,
	) -> Option<&Vec<UpdateEvent>> {
		self.updates.get(&connection_type)
	}

	pub fn get_mut_updates_for_connection_type(
		&mut self,
		connection_type: ConnectionType,
	) -> &Vec<UpdateEvent> {
		self.updates.entry(connection_type).or_default()
	}

	pub fn contains(&self, event: &UpdateEvent) -> bool {
		match self.updates.get(event.get_connection_type()) {
			Some(arr) => arr.contains(event),
			None => false,
		}
	}

	pub fn contains_complement(&self, event: &UpdateEvent) -> bool {
		self.contains(&event.get_complement())
	}

	fn remove(&mut self, event: &UpdateEvent) {
		if let Some(arr) = self.updates.get_mut(event.get_connection_type()) {
			arr.retain(|e| e.ne(event));
			if arr.is_empty() {
				self.updates.remove(event.get_connection_type());
			}
		}
	}

	fn add(&mut self, event: &UpdateEvent) {
		self.updates
			.entry(*event.get_connection_type())
			.or_default()
			.push(event.clone());
	}

	/// Clear out all pending updates
	fn clear(&mut self) {
		self.updates.clear()
	}
}

impl UpdateEvent {
	pub fn create_add(dsnp_user_id: DsnpUserId, connection_type: ConnectionType) -> Self {
		UpdateEvent::Add { dsnp_user_id, connection_type }
	}

	pub fn create_remove(dsnp_user_id: DsnpUserId, connection_type: ConnectionType) -> Self {
		UpdateEvent::Remove { dsnp_user_id, connection_type }
	}

	pub fn get_complement(&self) -> Self {
		match self {
			Add { dsnp_user_id, connection_type } =>
				Remove { dsnp_user_id: *dsnp_user_id, connection_type: *connection_type },
			Remove { dsnp_user_id, connection_type } =>
				Add { dsnp_user_id: *dsnp_user_id, connection_type: *connection_type },
		}
	}

	pub fn get_connection_type(&self) -> &ConnectionType {
		match self {
			Add { connection_type, .. } => connection_type,
			Remove { connection_type, .. } => connection_type,
		}
	}

	pub fn type_ordering(a: &UpdateEvent, b: &UpdateEvent) -> Ordering {
		match (a, b) {
			(Remove { .. }, Add { .. }) => Ordering::Less,
			(Add { .. }, Remove { .. }) => Ordering::Greater,
			_ => a.cmp(b),
		}
	}
}

pub trait UpdateAPI<E: EncryptionBehavior> {
	fn calculate_updates(
		&mut self,
		connection_keys: &Vec<DsnpKeys<E>>,
		encryption_key: (u64, &PublicKey<E>),
	) -> Result<Vec<ExportBundle>>;
}

impl<E: EncryptionBehavior> UpdateAPI<E> for UserGraph {
	fn calculate_updates(
		&mut self,
		connection_keys: &Vec<DsnpKeys<E>>,
		encryption_key: (u64, &PublicKey<E>),
	) -> Result<Vec<ExportBundle>> {
		self.calculate_updates(connection_keys, encryption_key)
	}
}

#[cfg(test)]
mod test {
	use super::*;
	use crate::dsnp::api_types::PrivacyType;

	#[test]
	fn tracker_register_should_return_error_for_duplicate_events() {
		// arrange
		let mut tracker = UpdateTracker::new();
		let event = UpdateEvent::create_add(1, ConnectionType::Follow(PrivacyType::Public));
		tracker
			.register_update(&event.clone())
			.expect("Should have registered successfully!");

		// act
		let exists = tracker.contains(&event);
		let res = tracker.register_update(&event);

		// assert
		assert!(exists);
		assert!(res.is_err());
	}

	#[test]
	fn tracker_register_should_remove_complement_events() {
		// arrange
		let mut tracker = UpdateTracker::new();
		let events = vec![
			UpdateEvent::create_add(1, ConnectionType::Follow(PrivacyType::Public)),
			UpdateEvent::create_remove(2, ConnectionType::Follow(PrivacyType::Public)),
		];
		tracker.register_updates(&events).expect("Should have registered successfully!");
		let complements: Vec<UpdateEvent> =
			events.as_slice().iter().map(|e| e.get_complement()).collect();

		// act
		let res = tracker.register_updates(&complements);

		// assert
		assert!(res.is_ok());
		assert_eq!(tracker.updates.len(), 0);
		assert!(!tracker.has_updates());
	}

	#[test]
	fn tracker_register_should_work_as_expected() {
		// arrange
		let mut tracker = UpdateTracker::new();
		let events = vec![
			UpdateEvent::create_add(1, ConnectionType::Follow(PrivacyType::Public)),
			UpdateEvent::create_remove(2, ConnectionType::Follow(PrivacyType::Public)),
			UpdateEvent::create_add(3, ConnectionType::Follow(PrivacyType::Private)),
			UpdateEvent::create_remove(4, ConnectionType::Follow(PrivacyType::Private)),
			UpdateEvent::create_add(5, ConnectionType::Friendship(PrivacyType::Public)),
			UpdateEvent::create_remove(6, ConnectionType::Friendship(PrivacyType::Public)),
			UpdateEvent::create_add(7, ConnectionType::Friendship(PrivacyType::Private)),
			UpdateEvent::create_remove(8, ConnectionType::Friendship(PrivacyType::Private)),
		];

		// act
		let res = tracker.register_updates(&events);

		// assert
		assert!(res.is_ok());

		let public_follow =
			tracker.updates.get(&ConnectionType::Follow(PrivacyType::Public)).unwrap();
		assert_eq!(public_follow.as_slice(), &events[..2]);

		let private_follow =
			tracker.updates.get(&ConnectionType::Follow(PrivacyType::Private)).unwrap();
		assert_eq!(private_follow.as_slice(), &events[2..4]);

		let public_friendship =
			tracker.updates.get(&ConnectionType::Friendship(PrivacyType::Public)).unwrap();
		assert_eq!(public_friendship.as_slice(), &events[4..6]);

		let private_friendship =
			tracker.updates.get(&ConnectionType::Friendship(PrivacyType::Private)).unwrap();
		assert_eq!(private_friendship.as_slice(), &events[6..8]);

		assert!(tracker.has_updates());
	}

	#[test]
	fn tracker_event_sorter_should_prioritize_removes() {
		// arrange
		let mut events = vec![
			UpdateEvent::create_add(1, ConnectionType::Follow(PrivacyType::Public)),
			UpdateEvent::create_remove(2, ConnectionType::Follow(PrivacyType::Public)),
		];

		// act
		events.sort_by(UpdateEvent::type_ordering);

		// assert
		assert_eq!(
			events,
			vec![
				UpdateEvent::create_remove(2, ConnectionType::Follow(PrivacyType::Public)),
				UpdateEvent::create_add(1, ConnectionType::Follow(PrivacyType::Public)),
			]
		)
	}
}
