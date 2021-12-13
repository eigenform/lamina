/// Structures for deliberate pointer-chasing.
///
/// [PointerMaze::shuffle] is an implementation of [Sattolo's
/// algorithm](https://en.wikipedia.org/wiki/Fisher%E2%80%93Yates_shuffle).
///

use std::convert::TryInto;
use crate::util::*;

/// Wrapper around a pointer.
#[derive(Clone, Copy, Debug)]
#[repr(transparent)]
pub struct Pointer(pub *const Self);
impl Default for Pointer {
    fn default() -> Self { Self(0 as *const Self) }
}

/// Storage for a cyclic chain of pointers.
///
/// The constant `SIZE` indicates the number of elements/pointers.
#[repr(C, align(4096))]
pub struct PointerMaze<const SIZE: usize> {
    pub data: Box<[Pointer]>, 
}
impl <const SIZE: usize> PointerMaze<SIZE> {

    /// Allocate a new object (on the heap) where all the members are pointers 
    /// initialized to zero.
    ///
    /// NOTE: You can't create a sized array and move it into a [Box] (you'll 
    /// run out of stack space with the big arrays we need here!) This whole 
    /// `.into_boxed_slice()` dance avoids those cases.
    pub fn new() -> Self {
        Self { 
            data: vec![Pointer::default(); SIZE]
                .into_boxed_slice().to_owned()
        }
    }

    /// Get a pointer to the first entry.
    pub fn head_ptr(&self) -> *const Pointer { &self.data[0] }
    /// Get a pointer to the middle entry.
    pub fn mid_ptr(&self) -> *const Pointer { &self.data[SIZE / 2] }
    /// Get a pointer to the last entry.
    pub fn tail_ptr(&self) -> *const Pointer { &self.data[SIZE - 1] }

    /// Return the size of the structure in bytes.
    pub fn size_in_bytes(&self) -> usize {
        std::mem::size_of::<[Pointer; SIZE]>()
    }
    /// Return the number of cache lines occupied by this structure.
    pub fn size_in_lines(&self) -> usize {
        self.size_in_bytes() / 64
    }
    /// Return the number of elements (pointers) in this structure.
    pub fn len(&self) -> usize {
        SIZE
    }

    /// Flush all associated cache lines.
    pub fn flush(&mut self) {
        let head = self.data.as_ptr() as *const [u8; 64];
        for line_idx in 0..self.size_in_lines() {
            unsafe { 
                let ptr = head.offset(
                    line_idx.try_into().unwrap()
                ) as *const u8;
                core::arch::x86_64::_mm_clflush(ptr);
            }
        }
    }

    /// Initialize each element with a pointer to itself.
    pub fn initialize(&mut self) {
        for idx in 0..SIZE {
            self.data[idx] = unsafe { 
                Pointer(self.data.as_ptr()
                    .offset(idx.try_into().unwrap()) 
                    as *const Pointer
                )
            };
        }
    }

    /// Shuffle elements, producing a randomized cyclic linked-list. 
    pub fn shuffle(&mut self, rng: &mut Xorshift64, stride: usize) {
        self.initialize();
        for i in (1..SIZE / stride).rev() {
            let j = rng.next() % i;
            let a = j * stride;
            let b = i * stride;
            self.data.swap(a, b);
        }
    }
}

