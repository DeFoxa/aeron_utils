use std::iter::FromIterator;
use std::{iter, slice, vec};

#[derive(PartialEq, Eq, PartialOrd, Hash, Clone, Copy, Debug)]
pub struct GenerationIndex {
    index: usize,
    generation: u64,
}

#[derive(Clone, Default)]
pub struct GenerationalIndexAllocator {
    entries: Vec<AllocatorEntry>,
    free: Vec<usize>,
}

#[derive(Clone, Default)]
pub struct GenerationalIndexArray<T>(Vec<Option<ArrayEntry<T>>>);

pub struct GenerationalIncexArrayIter<'a, T: 'a>(
    iter::Enumerate<slice::Iter<'a, Option<ArrayEntry<T>>>>,
);

pub struct GenerationalIndexArrayIterMut<'a, T: 'a>(
    iter::Enumerate<slice::IterMut<'a, Option<ArrayEntry<T>>>>,
);

pub struct GenerationalIndexArrayIntoIter<T>(iter::Enumerate<vec::IntoIter<Option<ArrayEntry<T>>>>);

#[derive(Clone)]
pub struct AllocatorEntry {
    is_live: bool,
    generation: u64,
}

#[derive(Clone)]
pub struct ArrayEntry<T> {
    value: T,
    generation: u64,
}

impl GenerationIndex {
    #[inline]
    pub fn index(&self) -> usize {
        self.index
    }

    #[inline]
    pub fn generation(&self) -> u64 {
        self.generation
    }
}
