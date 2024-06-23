use std::iter::FromIterator;
use std::{iter, slice, vec};

#[derive(PartialEq, Eq, PartialOrd, Hash, Clone, Copy, Debug)]
pub struct GenerationalIndex {
    index: usize,
    generation: u64,
}

impl GenerationalIndex {
    #[inline]
    pub fn index(&self) -> usize {
        self.index
    }

    #[inline]
    pub fn generation(&self) -> u64 {
        self.generation
    }
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

#[derive(Clone, Default)]
pub struct GenerationalIndexAllocator {
    entries: Vec<AllocatorEntry>,
    free: Vec<usize>,
}

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

impl GenerationalIndexAllocator {
    pub fn new() -> GenerationalIndexAllocator {
        Default::default()
    }

    pub fn allocate(&mut self) -> GenerationalIndex {
        if let Some(index) = self.free.pop() {
            let id_entry = &mut self.entries[index];
            assert!(!id_entry.is_live);
            GenerationalIndex {
                index,
                generation: id_entry.generation,
            }
        } else {
            self.entries.push(AllocatorEntry {
                is_live: true,
                generation: 0,
            });

            GenerationalIndex {
                index: self.entries.len() - 1,
                generation: 0,
            }
        }
    }

    pub fn deallocate(
        &mut self,
        generation_index: GenerationalIndex,
    ) -> Result<(), DeallocationError> {
        if generation_index.index >= self.entries.len() {
            return Err(DeallocationError::InvalidIndex);
        }

        let id_entry = &mut self.entries[generation_index.index];
        if !id_entry.is_live {
            return Err(DeallocationError::PreviouslyDeallocated);
        }

        id_entry.is_live = false;
        id_entry.generation = match id_entry.generation.checked_add(1) {
            Some(new_generation) => new_generation,
            None => return Err(DeallocationError::GenerationOverflow),
        };

        self.free.push(generation_index.index);

        Ok(())
    }

    #[inline]
    pub fn is_live(&self, generation_index: GenerationalIndex) -> bool {
        if generation_index.index < self.entries.len() {
            let id_entry = &self.entries[generation_index.index];
            id_entry.is_live && id_entry.generation == generation_index.generation
        } else {
            false
        }
    }

    #[inline]
    pub fn max_allocated_index(&self) -> usize {
        self.entries.len()
    }

    #[inline]
    pub fn live_at_index(&self, index: usize) -> Option<GenerationalIndex> {
        self.entries.get(index).and_then(|entry| {
            if entry.is_live {
                Some(GenerationalIndex {
                    index,
                    generation: self.entries[index].generation,
                })
            } else {
                None
            }
        })
    }
}

impl<T> GenerationalIndexArray<T> {
    pub fn new() -> GenerationalIndexArray<T> {
        GenerationalIndexArray(Vec::new())
    }
    pub fn clear(&mut self) {
        self.0.clear();
    }

    pub fn insert(
        &mut self,
        generation_index: GenerationalIndex,
        value: T,
    ) -> Option<(GenerationalIndex, T)> {
        if generation_index.index >= self.0.len() {
            for _ in self.0.len()..generation_index.index + 1 {
                self.0.push(None);
            }
        }

        let entry = &mut self.0[generation_index.index];

        let old = entry.take().map(|e| {
            (
                GenerationalIndex {
                    index: generation_index.index,
                    generation: e.generation,
                },
                e.value,
            )
        });
        *entry = Some(ArrayEntry {
            value,
            generation: generation_index.generation,
        });
        old
    }

    pub fn remove(&mut self, generation_index: GenerationalIndex) -> Option<T> {
        if generation_index.index < self.0.len() {
            let entry = &mut self.0[generation_index.index];

            if let Some(e) = entry.take() {
                if e.generation == generation_index.generation {
                    return Some(e.value);
                } else {
                    *entry = Some(e);
                }
            }
        }
        None
    }
}

pub enum DeallocationError {
    InvalidIndex,
    PreviouslyDeallocated,
    GenerationOverflow,
}
