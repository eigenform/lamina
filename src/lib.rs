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

use std::fs::File;
use std::io::Write;
use std::iter::FromIterator;
use std::collections::BTreeMap;

pub use dynasmrt::{
    dynasm, 
    DynasmApi, 
    DynasmLabelApi, 
    Assembler, 
    AssemblyOffset, 
    ExecutableBuffer, 
    x64::X64Relocation
};


/// Function pointer to emitted code which takes a pointer to an array of
/// [usize] used to hold the results.
///
/// FIXME: You should make the typing here more explicit to indicate that
/// we're passing a pointer to `[usize; 6]`.
pub type PMCTestFn = extern "C" fn(*mut usize);


/// Indicates the strategy used to obtain result values from emitted code.
pub enum PMCTestInterface {
    /// Emitted code stores 6 result values in some array. 
    ByRef(extern "C" fn(*mut [usize; 6])),
    /// Emitted code returns a single result value in RAX.
    ByVal(extern "C" fn() -> usize),
}

/// A set of results collected from PMCs.
pub struct PMCResults {
    /// The set of events counted.
    pub event: [Option<event::Event>; 6],
    /// Sets of data for each event.
    pub data: [Option<Vec<usize>>; 6], 

    pub min: [usize; 6],
    pub max: [usize; 6],
    pub map: [BTreeMap<usize, usize>; 6],

}
impl PMCResults {
    /// Create a new set of results.
    pub fn new(desc: &pmc::PerfCtlDescriptor) -> Self {
        use std::mem::MaybeUninit;
        const DATA: Option<Vec<usize>> = None;

        let maps: [BTreeMap<usize, usize>; 6] = unsafe {
            let mut maps: [MaybeUninit<BTreeMap<usize,usize>>; 6] = {
                MaybeUninit::uninit().assume_init()
            };
            for m in &mut maps {
                m.write(BTreeMap::new()); 
            }
            std::mem::transmute(maps)
        };
        let mut res = Self { 
            event: [None; 6], 
            data: [DATA; 6],
            min: [0; 6],
            max: [0; 6],
            map: maps,
        };
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
        if let Some(event) = &self.event[idx] {
            let mut dist = Vec::from_iter(&self.map[idx]);
            dist.sort_by(|&(_, a), &(_,b)| b.cmp(&a));
            let evt = format!("{:x?}", event);
            //println!("| --------------------------------------------------");
            println!("|  PMCx{:03x} [{}]", event.convert().0, evt);
            println!("|   Description:  {}", event.desc().desc);
            //println!("|   Counter type: {}", event.desc().unit.to_str());
            println!("|   min={:<5} max={:<5} mode={:<5} | dist={:?}",
                self.min[idx], self.max[idx], dist[0].0, self.map[idx]
            );
        }
    }

    /// Write raw result data to a text file
    pub fn write_txt(&self, name: &'static str) {
        let mut f = File::create(name).expect("cant create file");
        for idx in 0..6 {
            if let Some(event) = &self.event[idx] {
                let data = if let Some(data) = &self.data[idx] {
                    format!("{:?}", data)
                } else { unreachable!() };
                let line = format!("PMCx{:03x} {}|{}\n",
                    event.convert().0, event.desc().desc, data
                );
                f.write(line.as_bytes()).unwrap();
            }
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
    pub fn print(&self) {
        println!("# Test '{}'", self.name);
        for (idx, e) in self.res.event.iter().enumerate() {
            if e.is_some() {
                self.res.print_ctr(idx);
            }
        }
    }

    /// Run emitted code some number of times.
    ///
    /// # Safety
    /// This assumes that emitted code takes a pointer to an array of
    /// six 64-bit elements and fills it with the values of the six PMC
    /// counters (this is implicit in the definition of [PMCTestFn]. 
    ///
    /// Right now the only gadget satisfying this should be the 
    /// [emit_rdpmc_test_all] macro.
    ///
    /// Also note that this evicts code from the i-cache on each iteration.
    ///
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

        for idx in 0..6 {
            if let Some(ref mut data) = self.res.data[idx] {
                let mut map: BTreeMap<usize, usize> = BTreeMap::new();
                for value in data.iter() {
                    *map.entry(*value).or_insert(0) += 1;
                }
                let min = data.iter().min().unwrap();
                let max = data.iter().max().unwrap();
                self.res.map[idx] = map;
                self.res.min[idx] = *min;
                self.res.max[idx] = *max;
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

