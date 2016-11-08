use super::{ParallelIterator, ExactParallelIterator};

use std::collections::{BTreeMap, BTreeSet, HashMap, HashSet};
use std::hash::{BuildHasher, Hash};
use std::collections::LinkedList;
use std::collections::{BinaryHeap, VecDeque};

pub trait FromParallelIterator<PAR_ITER> {
    fn from_par_iter(par_iter: PAR_ITER) -> Self;
}


/// Collect an intermediate list of folded vectors.
fn collect_list_vec<PAR_ITER>(par_iter: PAR_ITER) -> LinkedList<Vec<PAR_ITER::Item>>
    where PAR_ITER: ParallelIterator
{
    par_iter.fold(Vec::new, |mut vec, elem| { vec.push(elem); vec })
            .collect()
}


/// Collect items from a parallel iterator into a freshly allocated
/// vector. This is very efficient, but requires precise knowledge of
/// the number of items being iterated.
impl<PAR_ITER, T> FromParallelIterator<PAR_ITER> for Vec<T>
    where PAR_ITER: ExactParallelIterator<Item=T>,
          T: Send,
{
    fn from_par_iter(par_iter: PAR_ITER) -> Self {
        let mut vec = vec![];
        par_iter.collect_into(&mut vec);
        vec
    }
}

/// Collect items from a parallel iterator into a vecdeque.
impl<PAR_ITER, T> FromParallelIterator<PAR_ITER> for VecDeque<T>
    where Vec<T>: FromParallelIterator<PAR_ITER>,
          T: Send,
{
    fn from_par_iter(par_iter: PAR_ITER) -> Self {
        Vec::from_par_iter(par_iter).into()
    }
}

/// Collect items from a parallel iterator into a binaryheap.
/// The heap-ordering is calculated serially after all items are collected.
impl<PAR_ITER, T> FromParallelIterator<PAR_ITER> for BinaryHeap<T>
    where Vec<T>: FromParallelIterator<PAR_ITER>,
          T: Ord + Send,
{
    fn from_par_iter(par_iter: PAR_ITER) -> Self {
        Vec::from_par_iter(par_iter).into()
    }
}

/// Collect items from a parallel iterator into a freshly allocated
/// linked list.
impl<PAR_ITER, T> FromParallelIterator<PAR_ITER> for LinkedList<T>
    where PAR_ITER: ParallelIterator<Item=T>,
          T: Send,
{
    fn from_par_iter(par_iter: PAR_ITER) -> Self {
        par_iter.map(|elem| { let mut list = LinkedList::new(); list.push_back(elem); list })
                .reduce_with(|mut list1, mut list2| { list1.append(&mut list2); list1 })
                .unwrap_or_else(|| LinkedList::new())
    }
}

/// Collect (key, value) pairs from a parallel iterator into a
/// hashmap. If multiple pairs correspond to the same key, then the
/// ones produced earlier in the parallel iterator will be
/// overwritten, just as with a sequential iterator.
impl<PAR_ITER, K, V, S> FromParallelIterator<PAR_ITER> for HashMap<K, V, S>
    where PAR_ITER: ParallelIterator<Item=(K, V)>,
          K: Eq + Hash + Send,
          V: Send,
          S: BuildHasher + Default + Send,
{
    fn from_par_iter(par_iter: PAR_ITER) -> Self {
        let list = collect_list_vec(par_iter);
        let len = list.iter().map(Vec::len).sum();
        list.into_iter()
            .fold(HashMap::with_capacity_and_hasher(len, Default::default()),
                  |mut map, vec| { map.extend(vec); map })
    }
}

/// Collect (key, value) pairs from a parallel iterator into a
/// btreemap. If multiple pairs correspond to the same key, then the
/// ones produced earlier in the parallel iterator will be
/// overwritten, just as with a sequential iterator.
impl<PAR_ITER, K, V> FromParallelIterator<PAR_ITER> for BTreeMap<K, V>
    where PAR_ITER: ParallelIterator<Item=(K, V)>,
          K: Ord + Send,
          V: Send,
{
    fn from_par_iter(par_iter: PAR_ITER) -> Self {
        let list = collect_list_vec(par_iter);
        list.into_iter()
            .fold(BTreeMap::new(),
                  |mut map, vec| { map.extend(vec); map })
    }
}

/// Collect values from a parallel iterator into a hashset.
impl<PAR_ITER, V, S> FromParallelIterator<PAR_ITER> for HashSet<V, S>
    where PAR_ITER: ParallelIterator<Item=V>,
          V: Eq + Hash + Send,
          S: BuildHasher + Default + Send,
{
    fn from_par_iter(par_iter: PAR_ITER) -> Self {
        let list = collect_list_vec(par_iter);
        let len = list.iter().map(Vec::len).sum();
        list.into_iter()
            .fold(HashSet::with_capacity_and_hasher(len, Default::default()),
                  |mut set, vec| { set.extend(vec); set })
    }
}

/// Collect values from a parallel iterator into a btreeset.
impl<PAR_ITER, V> FromParallelIterator<PAR_ITER> for BTreeSet<V>
    where PAR_ITER: ParallelIterator<Item=V>,
          V: Send + Ord,
{
    fn from_par_iter(par_iter: PAR_ITER) -> Self {
        let list = collect_list_vec(par_iter);
        list.into_iter()
            .fold(BTreeSet::new(),
                  |mut set, vec| { set.extend(vec); set })
    }
}
