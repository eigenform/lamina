/// Macros to help generate different microbenchmarks at runtime.

use crate::x86::*;
use dynasmrt::{
    dynasm, DynasmApi, DynasmLabelApi, 
    Assembler, AssemblyOffset, ExecutableBuffer, 
    x64::X64Relocation
};


#[macro_export]
macro_rules! emit_push_abi { ($asm:ident) => {
    dynasm!($asm
        ; .arch     x64
        ; push      rbx
        ; push      rdi
        ; push      rsi
        ; push      r12
        ; push      r13
        ; push      r14
        ; push      r15
        ; mfence
        ; xor       rbx, rbx
        ; xor       rdi, rdi
        ; xor       r12, r12
        ; xor       r13, r13
        ; xor       r14, r14
        ; xor       r15, r15
    );
}}

#[macro_export]
macro_rules! emit_pop_abi_ret { ($asm:ident) => {
    dynasm!($asm
        ; pop       r15
        ; pop       r14
        ; pop       r13
        ; pop       r12
        ; pop       rsi
        ; pop       rdi
        ; pop       rbx
        ; ret
    );
}}

/// Emit a bare loop using EAX [as the counter] and the JNE instruction.
#[macro_export]
macro_rules! emit_loop_eax { 
    ($asm:ident, $iters:expr, {$($body:tt)*}) => {
        dynasm!($asm
            ; mov       eax, $iters as _
            ; .align    64
            ; ->loop_head:
        );
        $($body)*
        dynasm!($asm
            ; sub       eax, 1
            ; jne       ->loop_head
        );
    }
}

/// Emit RDPRU, moving the whole 64-bit result into RDX (clobbering RAX).
///
/// ## Safety
/// This assumes that ECX is already prepared (0=MPERF, 1=APERF).
#[macro_export]
macro_rules! emit_rdpru_rdx { ($asm:ident, $($tail:tt)*) => {
    dynasm!($asm
        ; lfence
        ; .bytes    RDPRU
        ; lfence
        ; shl       rdx, 32
        ; or        rdx, rax
        $($tail)*
    );
}}


/// Emit a test using the RDPRU instruction to measure some code.
/// Returns the difference (number of cycles elapsed) in RAX.
///
/// - `to_rdi` and `to_rsi` are moved into RDI and RSI
/// - `loop_iters` specifies the number of loop iterations
/// - `outer_unroll` specifies how many times the head, body, and tail
///   elements are unrolled inside the loop
/// - Measured code is split into four groups of dynasm statements, which can 
///   be unrolled independently: `head`, `body_a`, `body_b`, and `tail`.
///
/// NOTE: You could probably make this a bit simpler ...
///
#[macro_export]
macro_rules! emit_aperf_simple {
    ($to_rdi:ident, $to_rsi:ident, $loop_iters:expr, $outer_unroll:expr,
       head($head_unroll:expr,   $($head:tt)*),
     body_a($body_a_unroll:expr, $($body_a:tt)*),
     body_b($body_b_unroll:expr, $($body_b:tt)*),
       tail($tail_unroll:expr,   $($tail:tt)*)
    ) => { {
        let mut asm = Assembler::<X64Relocation>::new().unwrap();
        emit_push_abi!(asm);
        dynasm!(asm
            // We have to keep 1 in RCX to read APERF with RDPRU
            // (this means measured code must not use RCX)
            ; mov       rcx, 1
            ; mov       rdi, QWORD $to_rdi as _
            ; mov       rsi, QWORD $to_rsi as _
            ; xor       r8, r8
        );

        // Take the first measurement and subtract from zero (in R8).
        emit_rdpru_rdx!(asm,
            ; sub       r8, rdx
        );

        emit_loop_eax!(asm, $loop_iters, {
            for _ in 0 ..$outer_unroll {
                for _ in 0..$head_unroll   { dynasm!(asm $($head)*  ); }
                for _ in 0..$body_a_unroll { dynasm!(asm $($body_a)*); }
                for _ in 0..$body_b_unroll { dynasm!(asm $($body_b)*); }
                for _ in 0..$tail_unroll   { dynasm!(asm $($tail)*  ); }
            }
        });

        // Take the second measurement (adding back to R8). 
        // APERF is a monotonically-increasing counter.
        emit_rdpru_rdx!(asm,
            ; add       r8, rdx
            ; mov       rax, r8
        );
        emit_pop_abi_ret!(asm);
        asm.finalize().unwrap()
    } }
}


