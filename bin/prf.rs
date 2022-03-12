use lamina::*;
use lamina::x86::*;
use lamina::util::*;
use lamina::chase::*;

/// The number of measurements taken per-test.
const SAMPLES: usize = 1024;

/// Number of times the gadget is unrolled within the loop.
const UNROLL: usize  = 256;

/// Number of loop iterations.
const ITER: usize    = 0x10;

fn main() {
    pin_to_core(0);

    let mut rng = Xorshift64::new();
    let mut mem = PointerMaze::<0x1000_0000>::new();
    let mut val = vec![1usize; 512].into_boxed_slice();
    mem.shuffle(&mut rng, 512);
    mem.flush();

    let ptr_a = mem.head_ptr() as *const usize;
    let ptr_b = mem.mid_ptr() as *const usize;
    let ptr_c = val.as_mut_ptr() as *const usize;

    for num_pad in 0..=256 {
        mem.flush();
        let mut res = [0usize; SAMPLES];

        // NOTE: You can reproduce this with other instructions that consume
        // entries in the integer PRF. This appears to measure the portion of 
        // [speculatively-consumed] physical register file entries.
        //
        // AMD's Software Optimization Guide (I'm looking at document 56305, 
        // Rev. 3.02, from March 2020) says the following in section 2.10.3
        // "Retire Control Unit", on page 34:
        //
        // > The retire control unit also manages internal integer register 
        // > mapping and renaming. The integer physical register file (PRF) 
        // > consists of 180 registers, with up to 38 per thread mapped to 
        // > architectural state or microarchitectural temporary state. The 
        // > remaining registers are available for out-of-order renames.
        //

        let test = emit_hwong_gadget_test!(
            ptr_a, ptr_b, ptr_c, ITER, UNROLL, num_pad,
            body_a(; add rax, r13),
            body_b(; add rax, r13)
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

