use std::{cell::UnsafeCell, mem::MaybeUninit};

// Don't touch pls am coding dis tmrw
/*
// Very lightweight slotmap implementation for minions
// Only supports u8 indices (only up to 64 tho), with no generational number (ids are not to be reused)
pub struct VeryDumbSlotMap<T> {
    storage: Vec<MaybeUninit<T>>,
    used_ids: u64,
}

impl<T> Default for VeryDumbSlotMap<T> {
    fn default() -> Self {
        Self { storage: Default::default(), used_ids: Default::default() }
    }
}

impl<T> VeryDumbSlotMap<T> {
    // Insert a new element into the dumb arena
    pub fn insert(&mut self, value: T) -> Option<u8> {
        let index = self.used_ids.trailing_ones();
        self.used_ids |= 1 << index;

        let new_len = self.storage.len().max(index as usize);
        self.storage.resize_with(new_len, || MaybeUninit::uninit());
        self.storage[index as usize] = MaybeUninit::new(value);
        Some(index as u8)
    }

    // Get an element immutably
    pub fn get(&self, index: u8) -> Option<&T> {
        if index >= 64 {
            return None;
        }
        
        if (self.used_ids >> index) & 1 == 0 {
            return None;
        }

        Some(unsafe { self.storage[index as usize].assume_init_ref() })
    }

    // Get an element mutably
    pub fn get_mut(&mut self, index: u8) -> Option<&mut T> {
        if index >= 64 {
            return None;
        }
        
        if (self.used_ids >> index) & 1 == 0 {
            return None;
        }

        Some(unsafe { self.storage[index as usize].assume_init_mut() })
    }

    // Remove an element using its id
    pub fn remove(&mut self, index: u8) {
        if (self.used_ids << index) & 1 != 1 {
            return;
        } 

        self.used_ids &= !(1 << index);
        unsafe {
            self.storage[index as usize].assume_init_drop()
        }
    }
}
*/