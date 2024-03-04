//! Implementation of transactional HashMap which tracks all changes before committing, and allows
//! rollbacks
use std::{
	borrow::Borrow,
	collections::{hash_map::Entry, HashMap},
	hash::Hash,
};

pub trait Transactional {
	fn commit(&mut self);
	fn rollback(&mut self);
}

#[derive(Debug, Eq, PartialEq, Clone)]
enum Reversible<K, V> {
	Add { key: K, prev: Option<V> },
	Remove { key: K, prev: V },
}

#[derive(Debug, Eq, PartialEq, Clone)]
pub struct TransactionalHashMap<K, V>
where
	K: Eq + Hash + Clone,
	V: Clone,
{
	inner: HashMap<K, V>,
	rollback_operations: Vec<Reversible<K, V>>,
}

impl<K, V> TransactionalHashMap<K, V>
where
	K: Eq + Hash + Clone,
	V: Clone,
{
	pub fn new() -> Self {
		Self { inner: HashMap::new(), rollback_operations: vec![] }
	}

	#[inline]
	pub fn with_capacity(capacity: usize) -> Self {
		Self { inner: HashMap::with_capacity(capacity), rollback_operations: vec![] }
	}

	pub fn inner(&self) -> &HashMap<K, V> {
		&self.inner
	}

	#[inline]
	pub fn entry(&mut self, key: K) -> Entry<'_, K, V> {
		match self.inner.entry(key.clone()) {
			Entry::Vacant(v) => {
				self.rollback_operations.push(Reversible::Add { prev: None, key });
				Entry::Vacant(v)
			},
			Entry::Occupied(o) => {
				self.rollback_operations
					.push(Reversible::Add { prev: Some(o.get().clone()), key });
				Entry::Occupied(o)
			},
		}
	}

	#[inline]
	pub fn insert(&mut self, k: K, v: V) -> Option<V> {
		let prev = self.inner.remove(&k);
		self.rollback_operations.push(Reversible::Add { prev, key: k.clone() });
		self.inner.insert(k, v)
	}

	#[inline]
	pub fn remove(&mut self, k: &K) -> Option<V> {
		if let Some(v) = self.inner.remove(&k) {
			self.rollback_operations
				.push(Reversible::Remove { prev: v.clone(), key: k.clone() });
			return Some(v);
		}
		None
	}

	#[inline]
	pub fn clear(&mut self) {
		for (key, value) in self.inner.iter() {
			self.rollback_operations
				.push(Reversible::Remove { key: key.clone(), prev: value.clone() });
		}

		self.inner.clear()
	}

	#[inline]
	pub fn len(&self) -> usize {
		self.inner.len()
	}

	#[inline]
	pub fn get<Q: ?Sized>(&self, k: &Q) -> Option<&V>
	where
		K: Borrow<Q>,
		Q: Hash + Eq,
	{
		self.inner.get(k)
	}

	#[inline]
	pub fn get_mut<Q: ?Sized>(&mut self, k: &Q) -> Option<&mut V>
	where
		K: Borrow<Q>,
		Q: Hash + Eq,
	{
		self.inner.get_mut(k)
	}
}

impl<K, V> FromIterator<(K, V)> for TransactionalHashMap<K, V>
where
	K: Eq + Hash + Clone,
	V: Clone,
{
	/// This is creating a new TransactionalHashMap from iterator and since it is initializing
	/// a new instance there is no need to track the initial items inside
	fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> TransactionalHashMap<K, V> {
		let mut map = TransactionalHashMap::new();
		map.extend(iter);
		map
	}
}

impl<K, V> Extend<(K, V)> for TransactionalHashMap<K, V>
where
	K: Eq + Hash + Clone,
	V: Clone,
{
	#[inline]
	fn extend<T: IntoIterator<Item = (K, V)>>(&mut self, iter: T) {
		self.inner.extend(iter)
	}
}

impl<K, V> Transactional for TransactionalHashMap<K, V>
where
	K: Eq + Hash + Clone,
	V: Clone,
{
	fn commit(&mut self) {
		self.rollback_operations = vec![];
	}

	fn rollback(&mut self) {
		while !self.rollback_operations.is_empty() {
			let op = self.rollback_operations.pop().unwrap();
			match op {
				Reversible::Add { prev, key } => match prev {
					Some(old) => self.inner.insert(key, old),
					None => self.inner.remove(&key),
				},
				Reversible::Remove { prev, key } => self.inner.insert(key, prev),
			};
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use std::collections::BTreeMap;

	#[test]
	fn transactional_hashmap_should_revert_the_state_as_before() {
		let arr = [(1, 9), (2, 8), (3, 7), (4, 6), (5, 5), (6, 4), (7, 3), (8, 2), (9, 1)];
		let mut transactional = TransactionalHashMap::new();
		for (k, v) in arr {
			transactional.insert(k, v);
		}
		transactional.commit();

		let arr2 = vec![(10, 100), (11, 200), (12, 300), (13, 400)];
		for (k, v) in arr2 {
			transactional.insert(k, v);
		}
		let inner_sorted: BTreeMap<_, _> = transactional.inner.clone().into_iter().collect();
		let expected = BTreeMap::from([
			(1, 9),
			(2, 8),
			(3, 7),
			(4, 6),
			(5, 5),
			(6, 4),
			(7, 3),
			(8, 2),
			(9, 1),
			(10, 100),
			(11, 200),
			(12, 300),
			(13, 400),
		]);
		assert_eq!(inner_sorted, expected);

		transactional.rollback();
		let inner_sorted: BTreeMap<_, _> = transactional.inner.clone().into_iter().collect();
		let expected = BTreeMap::from(arr);
		assert_eq!(inner_sorted, expected);

		transactional.remove(&5);
		transactional.insert(89, 98);
		let inner_sorted: BTreeMap<_, _> = transactional.inner.clone().into_iter().collect();
		let expected_2 = BTreeMap::from([
			(1, 9),
			(2, 8),
			(3, 7),
			(4, 6),
			(89, 98),
			(6, 4),
			(7, 3),
			(8, 2),
			(9, 1),
		]);
		assert_eq!(inner_sorted, expected_2);

		transactional.rollback();
		let inner_sorted: BTreeMap<_, _> = transactional.inner.clone().into_iter().collect();
		assert_eq!(inner_sorted, expected);

		assert_eq!(transactional.rollback_operations.len(), 0);
	}

	#[test]
	fn transactional_hashmap_entry_should_behave_as_expected() {
		let mut transactional = TransactionalHashMap::new();
		transactional.entry(5).and_modify(|n| *n = 100).or_insert(200);

		let inner_sorted: BTreeMap<_, _> = transactional.inner.clone().into_iter().collect();
		assert_eq!(inner_sorted, BTreeMap::from([(5, 200)]));
		transactional.commit();

		transactional.entry(5).and_modify(|n| *n = 100).or_insert(200);
		let inner_sorted: BTreeMap<_, _> = transactional.inner.clone().into_iter().collect();
		assert_eq!(inner_sorted, BTreeMap::from([(5, 100)]));

		transactional.rollback();
		let inner_sorted: BTreeMap<_, _> = transactional.inner.clone().into_iter().collect();
		assert_eq!(inner_sorted, BTreeMap::from([(5, 200)]));
	}
}
