use lamina::*;
use lamina::x86::*;
use lamina::util::*;
use lamina::chase::*;

use dynasmrt::{
    dynasm, DynasmApi, DynasmLabelApi, 
    Assembler, AssemblyOffset, ExecutableBuffer, 
    x64::X64Relocation
};

const SAMPLES: usize = 0x400;
const ITER: usize    = 0x100;

fn main() {
    pin_to_core(0);

    let mut mem = vec![0usize; 1024].into_boxed_slice();
    let ptr = mem.as_ptr();

    for x in 0..=0x2000 {
        let mut res = [0usize; SAMPLES];

        let test = emit_simple_loop_test!(ITER, ptr,
            head(
                ; mov [r15], 
                ; mov rax, [r15]
            ),
            body(x,
                ; nop
            ),
            tail(
                ; movnti [r15], r14
                ; mov rax, [r15]
            )
        );

        for i in 0..SAMPLES {
            res[i] = run_test(&test);
        }
        let min = *res.iter().min().unwrap() as f64
            / ITER as f64;
        let avg = res.iter().sum::<usize>() as f64
            / ITER as f64 / SAMPLES as f64;
        let max = *res.iter().max().unwrap() as f64
            / ITER as f64;
        println!("{:03}: min={:.3} avg={:.3} max={:.3}", x, min, avg, max);

    }


}
