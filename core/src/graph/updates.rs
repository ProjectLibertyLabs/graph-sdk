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
use log::Level;
use log_result_proc_macro::log_result_err;
use std::{cmp::Ordering, collections::HashSet};

/// Update event for a schema
#[derive(Clone, PartialEq, Ord, Eq, PartialOrd, Debug)]
pub enum UpdateEvent {
	/// Add event
	Add { dsnp_user_id: DsnpUserId, schema_id: SchemaId },
	/// Remove event
	Remove { dsnp_user_id: DsnpUserId, schema_id: SchemaId },
}

/// Update tracker for a  schema
#[derive(Debug, Eq, PartialEq, Clone)]
pub struct UpdateTracker {
	/// map of schema id to update events
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
	/// creates a new update tracker
	pub fn new() -> Self {
		Self { updates: TransactionalHashMap::new() }
	}

	/// registers an update event
	#[log_result_err(Level::Info)]
	pub fn register_update(
		&mut self,
		event: &UpdateEvent,
		ignore_existing: bool,
	) -> DsnpGraphResult<()> {
		if self.contains(event) {
			return match ignore_existing {
				true => Ok(()),
				false => Err(DsnpGraphError::EventExists),
			}
		}

		match self.contains_complement(event) {
			// removing the complement to cancel out a prior update
			true => self.remove(&event.get_complement()),
			// adding new update
			false => self.add(event),
		}

		Ok(())
	}

	/// registers multiple update events
	#[log_result_err(Level::Info)]
	pub fn register_updates(
		&mut self,
		events: &[UpdateEvent],
		ignore_existing: bool,
	) -> DsnpGraphResult<()> {
		if !ignore_existing && events.iter().any(|e| self.contains(e)) {
			return Err(DsnpGraphError::DuplicateUpdateEvents)
		}

		for e in events {
			self.register_update(e, ignore_existing)?;
		}

		Ok(())
	}

	/// returns true if there are any updates
	pub fn has_updates(&self) -> bool {
		self.updates.inner().iter().any(|(_, v)| !v.is_empty())
	}

	/// returns update events for the schema id
	pub fn get_updates_for_schema_id(&self, schema_id: SchemaId) -> Option<&Vec<UpdateEvent>> {
		self.updates.inner().get(&schema_id)
	}

	/// returns true if there are any updates for the schema id
	pub fn contains(&self, event: &UpdateEvent) -> bool {
		match self.updates.inner().get(event.get_schema_id()) {
			Some(arr) => arr.contains(event),
			None => false,
		}
	}

	/// returns true if there are any updates for the complement of the event
	pub fn contains_complement(&self, event: &UpdateEvent) -> bool {
		self.contains(&event.get_complement())
	}

	pub fn sync_updates(
		&mut self,
		schema_id: SchemaId,
		existing_connections: &HashSet<DsnpUserId>,
	) {
		if let Some(arr) = self.updates.get(&schema_id) {
			let mut synced_updates = arr.clone();
			synced_updates.retain(|e| match e {
				UpdateEvent::Add { dsnp_user_id, .. }
					if existing_connections.contains(&dsnp_user_id) =>
					false,
				UpdateEvent::Remove { dsnp_user_id, .. }
					if !existing_connections.contains(&dsnp_user_id) =>
					false,
				_ => true,
			});
			if synced_updates.len() != arr.len() {
				self.updates.insert(schema_id, synced_updates);
			}
		}
	}

	/// removes the update event
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

	/// adds the update event
	fn add(&mut self, event: &UpdateEvent) {
		self.updates.entry(*event.get_schema_id()).or_default().push(event.clone());
	}
}

impl UpdateEvent {
	/// creates an add event
	pub fn create_add(dsnp_user_id: DsnpUserId, schema_id: SchemaId) -> Self {
		UpdateEvent::Add { dsnp_user_id, schema_id }
	}

	/// creates a remove event
	pub fn create_remove(dsnp_user_id: DsnpUserId, schema_id: SchemaId) -> Self {
		UpdateEvent::Remove { dsnp_user_id, schema_id }
	}

	/// returns the complement of the event
	pub fn get_complement(&self) -> Self {
		match self {
			Add { dsnp_user_id, schema_id } =>
				Remove { dsnp_user_id: *dsnp_user_id, schema_id: *schema_id },
			Remove { dsnp_user_id, schema_id } =>
				Add { dsnp_user_id: *dsnp_user_id, schema_id: *schema_id },
		}
	}

	/// returns the schema id of the event
	pub fn get_schema_id(&self) -> &SchemaId {
		match self {
			Add { schema_id, .. } => schema_id,
			Remove { schema_id, .. } => schema_id,
		}
	}

	/// function to order the events
	/// removes are prioritized over adds
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
	fn tracker_register_should_handle_duplicate_events_as_requested() {
		// arrange
		let mut tracker = UpdateTracker::new();
		let schema_id = 4;
		let event = UpdateEvent::create_add(1, schema_id);
		tracker
			.register_update(&event.clone(), false)
			.expect("Should have registered successfully!");

		// act
		let exists = tracker.contains(&event);
		let res1 = tracker.register_update(&event, false);
		let res2 = tracker.register_update(&event, true);

		// assert
		assert!(exists);
		assert!(res1.is_err(), "should error on duplicate event");
		assert!(res2.is_ok(), "should ignore duplicate event");
		assert_eq!(
			tracker.get_updates_for_schema_id(schema_id).iter().count(),
			1,
			"should only contain original update event"
		);
	}

	#[test]
	fn tracker_register_should_remove_complement_events() {
		// arrange
		let mut tracker = UpdateTracker::new();
		let schema_id = 4;
		let events =
			vec![UpdateEvent::create_add(1, schema_id), UpdateEvent::create_remove(2, schema_id)];
		tracker
			.register_updates(&events, false)
			.expect("Should have registered successfully!");
		let complements: Vec<UpdateEvent> =
			events.as_slice().iter().map(|e| e.get_complement()).collect();

		// act
		let res = tracker.register_updates(&complements, false);

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
		let res = tracker.register_updates(&events, false);

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

	#[test]
	fn tracker_event_sync_updates_should_remove_redundant_events() {
		// arrange
		let mut tracker = UpdateTracker::new();
		let schema_1 = 4;
		let events = vec![
			UpdateEvent::create_add(1, schema_1),
			UpdateEvent::create_remove(2, schema_1),
			UpdateEvent::create_add(3, schema_1),
			UpdateEvent::create_remove(4, schema_1),
		];
		let existing_connections = HashSet::from([3, 2]);
		tracker.register_updates(&events, false).expect("should register");

		// act
		tracker.sync_updates(schema_1, &existing_connections);

		// assert
		let schema_1_events = tracker.updates.get(&schema_1).unwrap();
		assert_eq!(schema_1_events.as_slice(), &events[..2]);
	}
}
