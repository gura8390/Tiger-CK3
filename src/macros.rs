//! [`MacroCache`] to cache macro expansions, and [`MacroMap`] to track [`Loc`] use across macro expansions.

use std::hash::Hash;
use std::num::NonZeroU32;
use std::sync::{LazyLock, RwLock};

use crate::helpers::{BiTigerHashMap, TigerHashMap};
use crate::token::{Loc, Token};
use crate::tooltipped::Tooltipped;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
struct MacroKey {
    /// the loc of the call site
    loc: Loc,
    /// lexically sorted macro arguments
    args: Vec<(&'static str, &'static str)>,
    tooltipped: Tooltipped,
    /// only for triggers
    negated: bool,
}

impl MacroKey {
    pub fn new(
        loc: Loc,
        args: &[(&'static str, Token)],
        tooltipped: Tooltipped,
        negated: bool,
    ) -> Self {
        let mut args: Vec<_> = args.iter().map(|(parm, arg)| (*parm, arg.as_str())).collect();
        args.sort_unstable();
        Self { loc, args, tooltipped, negated }
    }
    fn recursion_key(mut self) -> (std::thread::ThreadId, Self) {
        self.loc.link_idx = None;
        (std::thread::current().id(), self)
    }
}

#[derive(Debug)]
/// A helper for scripted effects, triggers, and modifiers, all of which can
/// accept macro arguments and which need to be expanded for every macro call.
///
/// The cache helps avoid needless re-expansions for arguments that have already been validated.
pub struct MacroCache<T> {
    cache: RwLock<TigerHashMap<MacroKey, T>>,
    pending: RwLock<TigerHashMap<(std::thread::ThreadId, MacroKey), T>>,
}

impl<T> MacroCache<T> {
    pub fn perform<F: FnMut(&T)>(
        &self,
        key: &Token,
        args: &[(&'static str, Token)],
        tooltipped: Tooltipped,
        negated: bool,
        mut f: F,
    ) -> bool {
        let key = MacroKey::new(key.loc, args, tooltipped, negated);
        if let Some(x) = self.pending.read().unwrap().get(&key.clone().recursion_key()) {
            f(x);
            return true;
        }
        if let Some(x) = self.cache.read().unwrap().get(&key) {
            f(x);
            true
        } else {
            false
        }
    }

    pub fn insert(
        &self,
        key: &Token,
        args: &[(&'static str, Token)],
        tooltipped: Tooltipped,
        negated: bool,
        value: T,
    ) {
        let key = MacroKey::new(key.loc, args, tooltipped, negated);
        self.pending.write().unwrap().remove(&key.clone().recursion_key());
        self.cache.write().unwrap().insert(key, value);
    }
    /// Recursion sentinels belong to their worker; only completed results are
    /// shared. Completed keys retain the expansion call site for diagnostics.
    pub fn begin(&self, key: &Token, args: &[(&'static str, Token)], tooltipped: Tooltipped, negated: bool, value: T) {
        let key = MacroKey::new(key.loc, args, tooltipped, negated).recursion_key();
        self.pending.write().unwrap().insert(key, value);
    }
}

impl<T> Default for MacroCache<T> {
    fn default() -> Self {
        MacroCache { cache: RwLock::new(TigerHashMap::default()), pending: RwLock::new(TigerHashMap::default()) }
    }
}

/// Global macro map
pub(crate) static MACRO_MAP: LazyLock<MacroMap> = LazyLock::new(MacroMap::default);

#[derive(Default)]
pub struct MacroMap(RwLock<MacroMapInner>);

/// A bijective map storing the link index and the associated loc denoting the key
/// to the block containing the macros.
pub struct MacroMapInner {
    counter: NonZeroU32,
    bi_map: BiTigerHashMap<NonZeroU32, Loc>,
}

impl Default for MacroMapInner {
    fn default() -> Self {
        Self { counter: NonZeroU32::new(1).unwrap(), bi_map: BiTigerHashMap::default() }
    }
}

impl MacroMap {
    /// Get the loc associated with the index
    pub fn get_loc(&self, index: MacroMapIndex) -> Option<Loc> {
        self.0.read().unwrap().bi_map.get_by_left(&index.0).copied()
    }
    /// Get the index associated with the loc
    pub fn get_index(&self, loc: Loc) -> Option<MacroMapIndex> {
        self.0.read().unwrap().bi_map.get_by_right(&loc).copied().map(MacroMapIndex)
    }

    /// Insert a loc that is not expected to be in the map yet, and return its index.
    pub fn insert_or_get_loc(&self, loc: Loc) -> MacroMapIndex {
        let mut guard = self.0.write().unwrap();
        let counter = guard.counter;
        if guard.bi_map.insert_no_overwrite(counter, loc).is_err() {
            // The loc was already in the map. (The counter is always unique so that side can't have collided.)
            return guard.bi_map.get_by_right(&loc).copied().map(MacroMapIndex).unwrap();
        }
        guard.counter =
            guard.counter.checked_add(1).expect("internal error: 2^32 macro map entries");
        MacroMapIndex(counter)
    }

    /// Get the index of a loc, inserting it if it was not yet stored
    pub fn get_or_insert_loc(&self, loc: Loc) -> MacroMapIndex {
        // First try with just a read lock, which allows for more parallelism than using a write lock.
        if let Some(index) = self.get_index(loc) {
            index
        } else {
            // We need a write lock.
            self.insert_or_get_loc(loc)
        }
    }

    /// Clear all entries. This will break all existing `MacroMapIndex` values!
    pub(crate) fn clear(&self) {
        let mut guard = self.0.write().unwrap();
        guard.counter = NonZeroU32::new(1).unwrap();
        guard.bi_map.clear();
    }
}

/// Type-safety wrapper.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MacroMapIndex(NonZeroU32);

#[cfg(test)]
mod tests {
    use super::*;
    use crate::fileset::{FileKind, FileStage};
    #[test]
    fn incomplete_expansion_is_not_visible_to_another_worker() {
        let loc = Loc::for_file("cache-test.txt".into(), FileStage::NoStage, FileKind::Mod, "cache-test.txt".into());
        let key = Token::from_static_str("effect", loc);
        let cache = MacroCache::default();
        cache.begin(&key, &[], Tooltipped::No, false, 0);
        std::thread::scope(|s| {
            s.spawn(|| {
                assert!(!cache.perform(&key, &[], Tooltipped::No, false, |_| panic!("read another worker's provisional result")));
                cache.insert(&key, &[], Tooltipped::No, false, 42);
                assert!(cache.perform(&key, &[], Tooltipped::No, false, |value| assert_eq!(*value, 42)));
            }).join().unwrap();
        });
        assert!(cache.perform(&key, &[], Tooltipped::No, false, |value| assert_eq!(*value, 0)));
    }
}
