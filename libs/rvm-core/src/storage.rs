use ahash::{HashMap, HashMapExt};
use num_traits::ToPrimitive;
use num_traits::{NumCast, PrimInt};
use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt::{Debug, Display, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;

pub struct Storage<K: Hash + Eq + Debug, V: StorageValue> {
	lookup: HashMap<K, Id<V>>,
	values: Vec<V>,
}

impl<K: Hash + Eq + Debug, V: StorageValue> Storage<K, V> {
	pub fn new() -> Storage<K, V> {
		Storage {
			lookup: HashMap::new(),
			values: vec![],
		}
	}

	pub fn insert(&mut self, key: K, value: V) -> Id<V> {
		let mut idx = unsafe { Id::new(self.values.len() + 1) };
		if let Err(v) = self.lookup.try_insert(key, idx) {
			// replace value
			idx = *v.entry.get();
			*self.get_mut(idx) = value;
		} else {
			self.values.push(value);
		}
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

	pub fn get_keyed<Q: ?Sized>(&self, key: &Q) -> Option<&V>
	where
		K: Borrow<Q>,
		Q: Hash + Eq,
	{
		let id = self.get_id(key)?;
		Some(self.get(id))
	}

	pub fn get_mut_keyed<Q: ?Sized>(&mut self, key: &Q) -> Option<&mut V>
	where
		K: Borrow<Q>,
		Q: Hash + Eq,
	{
		let id = self.get_id(key)?;
		Some(self.get_mut(id))
	}

	pub fn get(&self, id: Id<V>) -> &V {
		unsafe { self.values.get_unchecked(id.0.to_usize().unwrap() - 1) }
	}

	pub fn get_mut(&mut self, id: Id<V>) -> &mut V {
		unsafe { self.values.get_unchecked_mut(id.0.to_usize().unwrap() - 1) }
	}

	pub fn iter(&self) -> &[V] {
		self.values.as_slice()
	}
}

pub struct Id<V: StorageValue>(V::Idx);

impl<V: StorageValue> Id<V> {
	pub unsafe fn new(idx: usize) -> Id<V> {
		Id((<V::Idx as NumCast>::from(idx)).unwrap())
	}

	pub fn idx(&self) -> V::Idx {
		self.0
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

pub trait StorageValue {
	type Idx: PrimInt + Hash + Display;
}

impl<V: StorageValue> StorageValue for Arc<V> {
	type Idx = V::Idx;
}
