//! Reimplementation of a gadget for measuring reorder buffer size; originally 
//! presented by Henry Wong in his blog entry "Measuring Reorder Buffer 
//! Capacity"[^1], and later reproduced by Travis Downs (see
//! [travisdowns/robsize](https://github.com/travisdowns/robsize) on GitHub).
//!
//! [^1]: ["stuffedcow.net - Measuring Reorder Buffer Capacity (2013)](https://blog.stuffedcow.net/2013/05/measuring-rob-capacity/)
//!

use lamina::*;
use lamina::codegen::*;
use lamina::x86::*;
use lamina::util::*;
use lamina::chase::*;

use dynasmrt::{
    dynasm, DynasmApi, DynasmLabelApi, 
    Assembler, AssemblyOffset, ExecutableBuffer, 
    x64::X64Relocation
};


fn main() {

    pin_to_core(0);

    // Create a random cyclic array of linked pointers, for deliberately
    // invoking loads that reliably miss in the L1 cache. 
    //
    // With a stride of 512 (8-byte pointers), each successive reference in
    // the chain should be separated by a page (512 * 8 = 4096 bytes).

    let mut rng = Xorshift64::new();
    let mut mem = PointerMaze::<0x2000000>::new();
    mem.shuffle(&mut rng, 512);
    mem.flush();

    let ptr_a = mem.head_ptr() as *const usize;
    let ptr_b = mem.mid_ptr() as *const usize;

    // Parameters for loop size/unrolling

    const RUNS: usize        = 64;
    const LOOP_UNROLL: usize = 16;
    const LOOP_ITER: usize   = 0x1000;

    // Emit and measure tests with 16-256 single-byte NOPs between loads.
    //
    // When both loads are not sharing the reorder buffer, they cannot be
    // paralellized, and the elapsed time should approximately double.
    //
    // On Zen 2, this reliably starts to act different at 223/224 NOPs
    // (this is the size of the reorder buffer on Zen 2 cores).

    for num_nops in 16..=256 {
        let mut res = [0usize; RUNS];

        let test = emit_aperf_simple!(ptr_a, ptr_b, LOOP_ITER, LOOP_UNROLL,
              head(1,        ; mov   rdi, [rdi]),
            body_a(num_nops, ; nop),
            body_b(1,        ; mov   rsi, [rsi]),
              tail(num_nops, ; nop)
        );

        for i in 0..RUNS {
            res[i] = run_test(&test);
        }

        let min = *res.iter().min().unwrap() as f64
            / LOOP_ITER as f64 / LOOP_UNROLL as f64;
        let avg = res.iter().sum::<usize>() as f64
            / LOOP_ITER as f64 / LOOP_UNROLL as f64 / RUNS as f64;
        let max = *res.iter().max().unwrap() as f64
            / LOOP_ITER as f64 / LOOP_UNROLL as f64 ;

        println!("{:03}: min={:.3} avg={:.3} max={:.3}", 
                 num_nops, min, avg, max);
    }
}

