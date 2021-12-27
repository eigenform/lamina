use lamina::*;
use lamina::x86::*;
use lamina::util::*;
use lamina::chase::*;

use dynasmrt::{
    dynasm, DynasmApi, DynasmLabelApi, 
    Assembler, AssemblyOffset, ExecutableBuffer, 
    x64::X64Relocation
};

/// The number of measurements taken per-test.
const SAMPLES: usize = 1024;

/// Number of times the gadget is unrolled within the loop.
const UNROLL: usize  = 128;

/// Number of loop iterations.
const ITER: usize    = 0x40;

fn main() {
    pin_to_core(0);

    let mut rng = Xorshift64::new();
    let mut mem = PointerMaze::<0x1000_0000>::new();
    let mut val = vec![0u8; 0x1000_0000].into_boxed_slice();

    mem.shuffle(&mut rng, 512);
    mem.flush();

    let ptr_a = mem.head_ptr() as *const usize;
    let ptr_b = mem.mid_ptr() as *const usize;
    let r15_ptr = val.as_ptr() as *const usize;

    for num_pad in 0..=64 {
        mem.flush();
        let mut res = [0usize; SAMPLES];

        // NOTE: It seems like the easiest way to do this is to do more loads
        // with RDI and RSI.

        let test = emit_hwong_gadget_test!(
            ptr_a, ptr_b, r15_ptr, ITER, UNROLL, num_pad,
            body_a(; mov rax, [rdi+64]),
            body_b(; mov rbx, [rsi+64])
        );

        for i in 0..SAMPLES {
            res[i] = run_test(&test);
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

