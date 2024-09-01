use std::borrow::Borrow;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use num_traits::ToPrimitive;
use num_traits::{Bounded, NumCast, PrimInt};

pub struct Storage<K: Hash + Eq + Debug, V: StorageValue, O = V> {
	lookup: HashMap<K, Id<V>>,
	values: Vec<O>,
}

impl<K: Hash + Eq + Debug, V: StorageValue, O> Storage<K, V, O> {
	pub fn new() -> Storage<K, V, O> {
		Storage {
			lookup: HashMap::new(),
			values: vec![],
		}
	}
	pub fn insert_keyed(&mut self, key: K, value: impl FnOnce(Id<V>) -> O) -> Id<V> {
		let mut idx = unsafe { Id::new(self.values.len() + 1) };
		if let Err(v) = self.lookup.try_insert(key, idx) {
			// replace value
			idx = *v.entry.get();
			*self.get_mut(idx) = value(idx);
		} else {
			self.values.push(value(idx));
		}
		idx
	}
	pub fn insert(&mut self, key: K, value: O) -> Id<V> {
		self.insert_keyed(key, |_| value)
	}

	pub fn push(&mut self, key: K, value: O) -> Id<V> {
		let idx = unsafe { Id::new(self.values.len() + 1) };
		if let Err(mut v) = self.lookup.try_insert(key, idx) {
			// replace value
			v.entry.insert(idx);
		}
		self.values.push(value);
		idx
	}

	pub fn contains(&self, key: &K) -> bool {
		self.lookup.contains_key(key)
	}

	pub fn contains_id(&self, id: Id<V>) -> bool {
		self.values.len() > id.0.to_usize().unwrap()
	}

	pub fn get_id<Q: ?Sized>(&self, key: &Q) -> Option<Id<V>>
	where
		K: Borrow<Q>,
		Q: Hash + Eq,
	{
		self.lookup.get(key).copied()
	}

	pub fn get_keyed<Q: ?Sized>(&self, key: &Q) -> Option<&O>
	where
		K: Borrow<Q>,
		Q: Hash + Eq,
	{
		let id = self.get_id(key)?;
		Some(self.get(id))
	}

	pub fn get_mut_keyed<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut O>
	where
		K: Borrow<Q>,
		Q: Hash + Eq,
	{
		let id = self.get_id(key)?;
		Some(self.get_mut(id))
	}

	pub fn get(&self, id: Id<V>) -> &O {
		&self.values[id.0.to_usize().unwrap() - 1]
		//unsafe { self.values.get_unchecked(id.0.to_usize().unwrap() - 1) }
	}

	pub fn get_mut(&mut self, id: Id<V>) -> &mut O {
		&mut self.values[id.0.to_usize().unwrap() - 1]
		//unsafe { self.values.get_unchecked_mut(id.0.to_usize().unwrap() - 1) }
	}

	pub fn iter(&self) -> &[O] {
		self.values.as_slice()
	}

	pub fn iter_keys_unordered(&self) -> impl Iterator<Item = (Id<V>, &K, &O)> {
		self.lookup
			.iter()
			.map(|(k, i)| (*i, k, &self.values[i.0.to_usize().unwrap() - 1]))
	}
}

pub struct Id<V: StorageValue>(V::Idx);

impl<V: StorageValue> Id<V> {
	pub unsafe fn new(idx: usize) -> Id<V> {
		Id((<V::Idx as NumCast>::from(idx)).unwrap())
	}

	pub fn null() -> Id<V> {
		Id(V::Idx::max_value())
	}

	pub fn idx(&self) -> V::Idx {
		self.0
	}
}
impl<K: Hash + Eq + Debug + Clone, V: StorageValue, O: Clone> Clone for Storage<K, V, O> {
	fn clone(&self) -> Self {
		Storage {
			lookup: self.lookup.clone(),
			values: self.values.clone(),
		}
	}
}

impl<V: StorageValue> Clone for Id<V> {
	fn clone(&self) -> Self {
		Id(self.0)
	}
}

impl<V: StorageValue> Debug for Id<V> {
	fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
		write!(f, "Class<{}>", self.0)
	}
}

impl<V: StorageValue> Copy for Id<V> {}

impl<V: StorageValue> PartialEq for Id<V> {
	fn eq(&self, other: &Self) -> bool {
		self.0.eq(&other.0)
	}
}

impl<V: StorageValue> Eq for Id<V> {}

impl<V: StorageValue> PartialOrd for Id<V> {
	fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
		self.0.partial_cmp(&other.0)
	}
}

impl<V: StorageValue> Ord for Id<V> {
	fn cmp(&self, other: &Self) -> Ordering {
		self.0.cmp(&other.0)
	}
}

impl<V: StorageValue> Hash for Id<V> {
	fn hash<H: Hasher>(&self, state: &mut H) {
		self.0.hash(state)
	}
}

pub trait StorageValue: Sized {
	type Idx: PrimInt + Hash + Display;
}

impl<V: StorageValue> StorageValue for Arc<V> {
	type Idx = V::Idx;
}
