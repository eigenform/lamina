#[allow(unused_macros)]

pub mod codegen;
pub mod util;
pub mod chase;
pub mod x86;

use std::convert::TryInto;
use dynasmrt::{ ExecutableBuffer, AssemblyOffset };

/// Call into a block of emitted code.
pub fn run_test(buf: &ExecutableBuffer) -> usize {
    let ptr: *const u8 = buf.ptr(AssemblyOffset(0));
    let line_ptr = ptr as *const [u8; 64];
    unsafe {
        let func: extern "C" fn() -> usize = std::mem::transmute(ptr);

        // Flush all emitted code from the caches
        for idx in 0..(buf.len() / 64) + 1 {
            core::arch::x86_64::_mm_clflush(
                line_ptr.offset(idx.try_into().unwrap()) as *const u8
            )
        }
        core::arch::x86_64::_mm_mfence();
        core::arch::x86_64::_mm_lfence();

        func()
    }
}


