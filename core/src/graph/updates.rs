#![allow(dead_code)]
use crate::{
	dsnp::dsnp_types::DsnpUserId,
	graph::updates::UpdateEvent::{Add, Remove},
	util::transactional_hashmap::{Transactional, TransactionalHashMap},
};
use dsnp_graph_config::{
	errors::{DsnpGraphError, DsnpGraphResult},
	SchemaId,
};
use std::cmp::Ordering;

#[derive(Clone, PartialEq, Ord, Eq, PartialOrd, Debug)]
pub enum UpdateEvent {
	Add { dsnp_user_id: DsnpUserId, schema_id: SchemaId },
	Remove { dsnp_user_id: DsnpUserId, schema_id: SchemaId },
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct UpdateTracker {
	updates: TransactionalHashMap<SchemaId, Vec<UpdateEvent>>,
}

/// implementing transactional trait for update tracker
impl Transactional for UpdateTracker {
	fn commit(&mut self) {
		self.updates.commit();
	}

	fn rollback(&mut self) {
		self.updates.rollback();
	}
}

impl UpdateTracker {
	pub fn new() -> Self {
		Self { updates: TransactionalHashMap::new() }
	}

	pub fn register_update(&mut self, event: &UpdateEvent) -> DsnpGraphResult<()> {
		if self.contains(event) {
			return Err(DsnpGraphError::EventExists)
		}

		match self.contains_complement(event) {
			// removing the complement to cancel out a prior update
			true => self.remove(&event.get_complement()),
			// adding new update
			false => self.add(event),
		}

		Ok(())
	}

	pub fn register_updates(&mut self, events: &[UpdateEvent]) -> DsnpGraphResult<()> {
		if events.iter().any(|e| self.contains(e)) {
			return Err(DsnpGraphError::DuplicateUpdateEvents)
		}

		for e in events {
			self.register_update(e)?;
		}

		Ok(())
	}

	pub fn has_updates(&self) -> bool {
		self.updates.inner().iter().any(|(_, v)| !v.is_empty())
	}

	pub fn get_updates_for_schema_id(&self, schema_id: SchemaId) -> Option<&Vec<UpdateEvent>> {
		self.updates.inner().get(&schema_id)
	}

	pub fn contains(&self, event: &UpdateEvent) -> bool {
		match self.updates.inner().get(event.get_schema_id()) {
			Some(arr) => arr.contains(event),
			None => false,
		}
	}

	pub fn contains_complement(&self, event: &UpdateEvent) -> bool {
		self.contains(&event.get_complement())
	}

	fn remove(&mut self, event: &UpdateEvent) {
		if let Some(arr) = self.updates.get(event.get_schema_id()) {
			let mut updates = arr.clone();
			updates.retain(|e| e.ne(event));
			match updates.is_empty() {
				true => self.updates.remove(event.get_schema_id()),
				false => self.updates.insert(*event.get_schema_id(), updates),
			};
		}
	}

	fn add(&mut self, event: &UpdateEvent) {
		self.updates.entry(*event.get_schema_id()).or_default().push(event.clone());
	}
}

impl UpdateEvent {
	pub fn create_add(dsnp_user_id: DsnpUserId, schema_id: SchemaId) -> Self {
		UpdateEvent::Add { dsnp_user_id, schema_id }
	}

	pub fn create_remove(dsnp_user_id: DsnpUserId, schema_id: SchemaId) -> Self {
		UpdateEvent::Remove { dsnp_user_id, schema_id }
	}

	pub fn get_complement(&self) -> Self {
		match self {
			Add { dsnp_user_id, schema_id } =>
				Remove { dsnp_user_id: *dsnp_user_id, schema_id: *schema_id },
			Remove { dsnp_user_id, schema_id } =>
				Add { dsnp_user_id: *dsnp_user_id, schema_id: *schema_id },
		}
	}

	pub fn get_schema_id(&self) -> &SchemaId {
		match self {
			Add { schema_id, .. } => schema_id,
			Remove { schema_id, .. } => schema_id,
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

#[cfg(test)]
mod test {
	use super::*;

	#[test]
	fn tracker_register_should_return_error_for_duplicate_events() {
		// arrange
		let mut tracker = UpdateTracker::new();
		let schema_id = 4;
		let event = UpdateEvent::create_add(1, schema_id);
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
		let schema_id = 4;
		let events =
			vec![UpdateEvent::create_add(1, schema_id), UpdateEvent::create_remove(2, schema_id)];
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
		let schema_1 = 4;
		let schema_2 = 5;

		let events = vec![
			UpdateEvent::create_add(1, schema_1),
			UpdateEvent::create_remove(2, schema_1),
			UpdateEvent::create_add(3, schema_2),
			UpdateEvent::create_remove(4, schema_2),
		];

		// act
		let res = tracker.register_updates(&events);

		// assert
		assert!(res.is_ok());

		let schema_1_events = tracker.updates.get(&schema_1).unwrap();
		assert_eq!(schema_1_events.as_slice(), &events[..2]);

		let schema_2_events = tracker.updates.get(&schema_2).unwrap();
		assert_eq!(schema_2_events.as_slice(), &events[2..4]);

		assert!(tracker.has_updates());
	}

	#[test]
	fn tracker_event_sorter_should_prioritize_removes() {
		// arrange
		let schema_id = 4;
		let mut events =
			vec![UpdateEvent::create_add(1, schema_id), UpdateEvent::create_remove(2, schema_id)];

		// act
		events.sort_by(UpdateEvent::type_ordering);

		// assert
		assert_eq!(
			events,
			vec![UpdateEvent::create_remove(2, schema_id), UpdateEvent::create_add(1, schema_id),]
		)
	}
}
