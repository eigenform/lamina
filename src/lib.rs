#[allow(unused_macros)]

pub mod codegen;
pub mod util;
pub mod chase;
pub mod x86;

use dynasmrt::{ ExecutableBuffer, AssemblyOffset };

/// Call into a block of emitted code.
pub fn run_test(buf: &ExecutableBuffer) -> usize {
    let ptr: *const u8 = buf.ptr(AssemblyOffset(0));
    unsafe {
        let func: extern "C" fn() -> usize = std::mem::transmute(ptr);
        core::arch::x86_64::_mm_clflush(ptr);
        core::arch::x86_64::_mm_mfence();
        func()
    }
}


