//! Reimplementation of a gadget for measuring reorder buffer size; originally 
//! presented by Henry Wong in his blog entry "Measuring Reorder Buffer 
//! Capacity"[^1], and later reproduced by Travis Downs[^2].
//!
//! ## How does this work?
//! In essence, H. Wong's gadget looks like this, where `body_a` and
//! `body_b` are filled with some number of padding instructions:
//!
//!          mov rC, <loop iterations>
//!
//!      loop:
//!          mov rA, [rA] # High-latency load 1
//!          { body_a } 
//!          mov rB, [rB] # High-latency load 2
//!          { body_b } 
//!
//!          sub rC, 1
//!          jne loop
//!          ...
//!
//! The chain of pointer derefences with rA and rB are a way to ensure that 
//! each of the two loads take a very long time. The loop (and optionally,
//! unrolling in the inner part of the loop) are not strictly necessary,
//! but they make the timing differences much easier to observe.
//!
//! A reorder buffer is basically the largest window of instructions whose
//! work can be parallelized. Since the machine makes effects visible in 
//! program order, high-latency instructions cause younger instructions in 
//! the stream to wait before they are retired, even though the underlying 
//! work associated with the instruction is parallelized.
//!
//! Assume we have a ROB that can track up to 4 instructions:
//!
//!      mov rA, [rA]    | # ROB state
//!      nop             |
//!      mov rB, [rB]    |
//!      nop             |
//!      ...
//!
//! NOP instructions are allocated an entry in the reorder buffer.
//! By steadily increasing the number of padding NOPs between loads, we
//! will eventually create a situation where both of the high-latency loads 
//! cannot share the reorder buffer (because we've filled it with NOPs).
//! 
//!      mov rA, [rA]    | # ROB state
//!      nop             |
//!      nop             |
//!      nop             |
//!      ...
//!      mov rB, [rB]    | # ROB state
//!      nop             |
//!      nop             |
//!      nop             |
//!      ...
//!
//! When both high-latency loads are not sharing the reorder buffer, they 
//! cannot be paralellized, and the number of cycles elapsed should 
//! approximately double! On Zen 2, this reliably starts to act differently 
//! around ~224 NOPs (this is the published size of the reorder buffer on 
//! Zen 2 cores).
//!
//! [^1]: ["stuffedcow.net - Measuring Reorder Buffer Capacity (2013)](https://blog.stuffedcow.net/2013/05/measuring-rob-capacity/)
//!
//! [^2]: [travisdowns/robsize](https://github.com/travisdowns/robsize)
//!

use lamina::*;
use lamina::x86::*;
use lamina::util::*;
use lamina::chase::*;

/// The number of measurements taken per-test.
const SAMPLES: usize = 512;

/// Number of times the gadget is unrolled within the loop.
const UNROLL: usize  = 32;

/// Number of loop iterations.
const ITER: usize    = 0x80;

fn main() {

    // NOTE: You probably want to run this with simultaneous multithreading 
    // (SMT) disabled, so that we always schedule this process on a single 
    // hardware thread. See [scripts/config-cpu].

    pin_to_core(0);

    // Create a random cyclic array of linked pointers, for deliberately
    // invoking loads that reliably miss in the L1 cache. 
    //
    // With a stride of 512 (8-byte pointers), each successive reference in
    // the chain should be separated by a page (512 * 8 = 4096 bytes).

    let mut rng = Xorshift64::new();
    let mut mem = PointerMaze::<0x1000_0000>::new();
    mem.shuffle(&mut rng, 512);
    mem.flush();

    let ptr_a = mem.head_ptr() as *const usize;
    let ptr_b = mem.mid_ptr() as *const usize;
    let ptr_c = 0 as *const usize;

    for num_pad in 0..=256 {
        mem.flush();
        let mut res = [0usize; SAMPLES];

        // This is a macro that generates the gadget; see [src/codegen.rs]
        let test = emit_hwong_gadget_test!(
            ptr_a, ptr_b, ptr_c, ITER, UNROLL, num_pad,
            body_a(; nop),
            body_b(; nop)
        );

        for i in 0..SAMPLES {
            res[i] = run_simple_test(&test);
        }

        let min = *res.iter().min().unwrap() as f64
            / ITER as f64 / UNROLL as f64;
        let avg = res.iter().sum::<usize>() as f64
            / ITER as f64 / UNROLL as f64 / SAMPLES as f64;
        let max = *res.iter().max().unwrap() as f64
            / ITER as f64 / UNROLL as f64 ;

        println!("{:03}: min={:.3} avg={:.3} max={:.3}", 
                 num_pad, min, avg, max);
    }
}

