//! Implementation of transactional Vec which tracks all changes before committing, and allows
//! rollbacks
use crate::util::transactional_hashmap::Transactional;

#[derive(Debug, Eq, PartialEq, Clone)]
enum Reversible<T> {
	Add { index: usize },
	Remove { index: usize, value: T },
}

#[derive(Debug, PartialEq, Clone)]
pub struct TransactionalVec<T>
where
	T: Clone,
{
	inner: Vec<T>,
	rollback_operations: Vec<Reversible<T>>,
}

impl<T> TransactionalVec<T>
where
	T: Clone,
{
	pub fn new() -> Self {
		Self { inner: Vec::new(), rollback_operations: vec![] }
	}

	/// This is creating a new TransactionalVec from iterator and since it is initializing
	/// a new instance there is no need to track the initial items inside
	pub fn from(inner: Vec<T>) -> Self {
		Self { inner, rollback_operations: vec![] }
	}

	pub fn inner(&self) -> &Vec<T> {
		&self.inner
	}

	#[inline]
	pub fn len(&self) -> usize {
		self.inner.len()
	}

	#[inline]
	pub fn clear(&mut self) {
		for (index, val) in self.inner.iter().enumerate().rev() {
			self.rollback_operations.push(Reversible::Remove { index, value: val.clone() });
		}

		self.inner.clear()
	}

	#[inline]
	pub fn push(&mut self, value: T) {
		self.rollback_operations.push(Reversible::Add { index: self.inner.len() });
		self.inner.push(value)
	}

	pub fn retain<F>(&mut self, mut f: F)
	where
		F: FnMut(&T) -> bool,
	{
		for (index, val) in self.inner.iter().enumerate().rev() {
			if !f(val) {
				self.rollback_operations.push(Reversible::Remove { index, value: val.clone() });
			}
		}
		self.inner.retain(f);
	}

	pub fn extend_from_slice(&mut self, other: &[T]) {
		let mut index = self.inner.len();
		for _ in other {
			self.rollback_operations.push(Reversible::Add { index });
			index += 1;
		}
		self.inner.extend_from_slice(other)
	}
}

impl<T> Transactional for TransactionalVec<T>
where
	T: Clone,
{
	fn commit(&mut self) {
		self.rollback_operations = vec![];
	}

	fn rollback(&mut self) {
		while !self.rollback_operations.is_empty() {
			let op = self.rollback_operations.pop().unwrap();
			match op {
				Reversible::Add { index } => {
					self.inner.remove(index);
				},
				Reversible::Remove { index, value } => {
					self.inner.insert(index, value);
				},
			};
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn transactional_vec_should_revert_the_state_as_before_using_clean_and_extend() {
		let arr = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
		let mut transactional = TransactionalVec::from(arr.clone());

		let arr2 = vec![10, 11, 12, 13];
		transactional.extend_from_slice(&arr2);
		assert_eq!(transactional.inner, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13]);

		transactional.rollback();
		assert_eq!(transactional.inner, arr);

		transactional.clear();
		assert_eq!(transactional.inner, Vec::<i32>::new());
		transactional.extend_from_slice(&vec![100, 89, 8]);

		transactional.rollback();
		assert_eq!(transactional.inner, arr);

		let arr3 = vec![9, 3, 5, 6];
		transactional.clear();
		transactional.extend_from_slice(&arr3);
		transactional.commit();
		assert_eq!(transactional.inner, arr3);
	}

	#[test]
	fn transactional_vec_should_revert_the_state_as_before_using_push_and_retain() {
		let arr = vec![1, 2, 3, 4, 5, 6, 7, 8, 9];
		let mut transactional = TransactionalVec::from(arr.clone());

		let arr2 = vec![10, 11, 12, 13];
		arr2.iter().for_each(|i| transactional.push(*i));
		assert_eq!(transactional.inner, vec![1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13]);

		transactional.rollback();
		assert_eq!(transactional.inner, arr);

		transactional.retain(|i| *i < 5);
		assert_eq!(transactional.inner, vec![1, 2, 3, 4]);
		transactional.push(1000);

		transactional.rollback();
		assert_eq!(transactional.inner, arr);
	}
}
