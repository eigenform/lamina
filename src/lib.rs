//! Library with tools for writing and running microbenchmarks.
//!
//! Note that this crate re-exports parts of [dynasmrt].
//!
//! ## Safety
//!
//! This is all very unsafe because most of this involves generating code 
//! (and potentially *weird* code) at runtime, but you probably already knew 
//! that. Right now, many parts of this library are not necessarily meant to 
//! provide a *totally* safe and painless interface. Proceed with caution.
//!
//! ## Kernel module
//!
//! A Linux kernel module `lamina.ko` provides a mechanism for writing the
//! PMCs from userspace. You need to use this if you're planning on writing
//! tests that utilize the `RDPMC` instruction. 
//!
//! You can use the `Makefile` in the project root to build the module. 
//! Additionally, note that it's **required** that you run 
//! `scripts/config-cpu enable` before loading the kernel module and using it. 
//! The script may not map cleanly to the situation on your machine, so you 
//! should probably read it before running it.
//!
//! ## Usage (simple tests without PMCs)
//!
//! For running code that doesn't rely on the kernel module and RDPMC usage,
//! you can use [run_simple_test]. 
//!
//! See examples in the `bin/` directory.
//!
//! ## Usage (tests that issue RDPMC)
//!
//! ```
//! use lamina::*;
//! use lamina::ctx::PMCContext;
//! use lamina::pmc::PerfCtlDescriptor;
//! use lamina::event::Event;
//!
//! fn main() -> Result<(), &'static str> {
//!     // The kernel module always instruments PMCs on core 0
//!     lamina::util::pin_to_core(0);
//!
//!     // This is the interface to the kernel module 
//!     let mut ctx = PMCContext::new()?;
//!
//!     // Measure retired instructions with counter 0.
//!     let mut pmc = PerfCtlDescriptor::new()
//!         .set(0, Event::ExRetInst(0), true);
//!
//!     // Create a test
//!     let code = emit_rdpmc_test_all!(
//!         ; nop ; nop ; nop; nop
//!     );
//!     let mut test = PMCTest::new("4 nops", &code, &pmc);
//!
//!     // Enable counters for the selected events
//!     ctx.write(&pmc);
//!
//!     // Run the test and collect results
//!     test.run_iter(0x1000);
//!
//!     // Do some analysis on the results
//!     // ...
//!
//!     Ok(())
//! }
//! ```
//!
//! You can see more examples in `bin/pmc/` directory.
//!

#[allow(unused_macros)]

pub mod codegen;
pub mod util;
pub mod chase;
pub mod x86;
pub mod pmc;
pub mod event;
pub mod ctx;

pub use dynasmrt::{
    dynasm, 
    DynasmApi, 
    DynasmLabelApi, 
    Assembler, 
    AssemblyOffset, 
    ExecutableBuffer, 
    x64::X64Relocation
};


/// Function pointer to emitted code (using PMCs).
pub type PMCTestFn = extern "C" fn(*mut usize);

/// A set of results collected from PMCs.
pub struct PMCResults {
    /// The set of events counted.
    pub event: [Option<event::Event>; 6],
    /// Sets of data for each event.
    pub data: [Option<Vec<usize>>; 6], 
}
impl PMCResults {
    /// Create a new set of results.
    pub fn new(desc: &pmc::PerfCtlDescriptor) -> Self {
        const DATA: Option<Vec<usize>> = None;
        let mut res = Self { event: [None; 6], data: [DATA; 6] };
        for idx in 0..6 {
            res.event[idx] = desc.events[idx];
            if let Some(_) = desc.events[idx] {
                res.data[idx] = Some(Vec::new());
            }
        }
        res
    }
    pub fn print_ctr(&self, idx: usize) {
        assert!(idx < 6);
        if let Some(data) = &self.data[idx] {
            let evt = self.event[idx].unwrap();
            let min = data.iter().min().unwrap();
            let max = data.iter().max().unwrap();
            println!("{:x?} min={} max={}", evt, min, max);
        }
    }
}

/// Wrapper around emitted code that uses RDPMC to capture some data.
pub struct PMCTest {
    /// User-provided description for this test.
    pub name: &'static str,
    /// Size of emitted code in bytes.
    pub size: usize,
    /// Pointer to emitted code.
    pub ptr: *const u8,
    /// Function pointer for emitted code.
    pub func: PMCTestFn,
    /// The latest set of result data from this test.
    pub res: PMCResults,
}
impl PMCTest {
    /// Create a new test.
    pub fn new(name: &'static str, buf: &ExecutableBuffer, 
        desc: &pmc::PerfCtlDescriptor
    ) -> Self {
        let ptr: *const u8 = buf.ptr(AssemblyOffset(0));
        unsafe {
            Self { 
                name,
                ptr,
                size: buf.len(),
                func: std::mem::transmute(ptr),
                res: PMCResults::new(desc),
            }
        }
    }

    /// Run the emitted code once.
    pub fn run_once(&self) -> [usize; 6] { 
        let mut res: [usize; 6] = [0; 6];
        (self.func)(res.as_mut_ptr()); 
        res
    }

    /// Run emitted code some number of times.
    pub fn run_iter(&mut self, iter: usize) {
        let mut res_vec = vec![[0usize;6]; iter];

        for i in 0..iter { 
            let mut res: [usize; 6] = [0; 6];
            util::clflush(self.size, self.ptr as *const [u8; 64]);
            (self.func)(res.as_mut_ptr());
            res_vec[i] = res;
        }

        for res in res_vec.iter() {
            for idx in 0..6 {
                if let Some(ref mut v) = self.res.data[idx] { 
                    v.push(res[idx]);
                }
            }
        }
    }
}


/// Function pointer to emitted code (no PMC usage).
pub type SimpleTestFn = extern "C" fn() -> usize;

/// Call into a block of emitted code.
///
/// Converts an [ExecutableBuffer] into [SimpleTestFn] and executes it.
pub fn run_simple_test(buf: &ExecutableBuffer) -> usize {
    let ptr: *const u8 = buf.ptr(AssemblyOffset(0));
    unsafe {
        let func: SimpleTestFn = std::mem::transmute(ptr);
        util::clflush(buf.len(), ptr as *const [u8; 64]);
        func()
    }
}

