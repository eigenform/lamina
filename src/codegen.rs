//! Macros for generating different microbenchmarks at runtime.
//!
//! These are mostly all wrappers around [dynasm] macros, used for easily 
//! parameterizing different kinds of gadgets/idioms in assembly.
//!
//! ## Compatibility
//! The behavior of some of the assembly here might depend on the way that
//! the Zen 2 microarchitecture is implemented (written and tested only on
//! a Ryzen 9 3950X). You might also see AMD-specific instructions.
//!
//! When calling into emitted code, we follow the SysV ABI (written and tested
//! only on 64-bit Linux).
//!

use crate::x86::*;
use dynasmrt::{
    dynasm, DynasmApi, DynasmLabelApi, 
    Assembler, AssemblyOffset, ExecutableBuffer, 
    x64::X64Relocation
};

/// Common prologue for emitted code, clearing all of the general-purpose 
/// registers (with the exception of RSP).
///
/// NOTE: Stack usage in emitted code is unsupported right now (for no reason 
/// in particular).
#[macro_export]
macro_rules! emit_push_abi { ($asm:ident) => {
    dynasm!($asm
        ; .arch     x64
        // Callee-save registers (in the SysV ABI)
        ; push      rbp
        ; push      rbx
        ; push      rdi
        ; push      rsi
        ; push      r12
        ; push      r13
        ; push      r14
        ; push      r15
        ; mfence
        ; lfence
        // Clear all of the general-purpose registers
        ; xor       rax, rax
        ; xor       rbx, rbx
        ; xor       rcx, rcx
        ; xor       rdx, rdx
        ; xor       rsi, rsi
        ; xor       rdi, rdi
        ; xor       rbp, rbp
        ; xor        r8, r8
        ; xor        r9, r9
        ; xor       r10, r10
        ; xor       r11, r11
        ; xor       r12, r12
        ; xor       r13, r13
        ; xor       r14, r14
        ; xor       r15, r15
        ; lfence
    );
}}

/// Common epilogue for emitted code.
#[macro_export]
macro_rules! emit_pop_abi_ret { ($asm:ident) => {
    dynasm!($asm
        // Callee-save registers (in the SysV ABI)
        ; pop       r15
        ; pop       r14
        ; pop       r13
        ; pop       r12
        ; pop       rsi
        ; pop       rdi
        ; pop       rbx
        ; pop       rbp
        ; ret
        ; lfence
    );
}}

/// Emit a bare loop using some register `$reg` and the JNE instruction.
#[macro_export]
macro_rules! emit_loop_reg { 
    ($asm:ident, $reg:tt, $iters:expr, {$($body:tt)*}) => {
        dynasm!($asm
            ; mov       $reg, $iters as _
            ; .align    64
            ; ->loop_head:
            ; lfence
        );
        $($body)*
        dynasm!($asm
            ; sub       $reg, 1
            ; jne       ->loop_head
            ; lfence
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


#[macro_export]
macro_rules! emit_simple_loop_test {
    ($iters:expr, $ptr:ident,
     head($($head:tt)*), 
     body($rept:expr, $($body:tt)*),
     tail($($tail:tt)*)
     ) => {{
        let mut asm = Assembler::<X64Relocation>::new().unwrap();
        emit_push_abi!(asm);
        dynasm!(asm
            ; xor r14, r14
            ; mov r15, QWORD $ptr as _
        );

        emit_rdpru_rdx!(asm, ; sub r14, rdx);

        emit_loop_reg!(asm, rdx, $iters, {
            dynasm!(asm $($head)*);
            for _ in 0..$rept {
                dynasm!(asm $($body)*);
            }
            dynasm!(asm $($tail)*);
        });

        emit_rdpru_rdx!(asm, ; add r14, rdx; mov rax, r14);

        emit_pop_abi_ret!(asm);
        asm.finalize().unwrap()
    }}
}


/// Generator for variations on Henry Wong's gadget for measuring reorder 
/// buffer capacity. 
///
/// This assumes that `tgt_ptr1` and `tgt_ptr2` are both pointers in some 
/// cyclic, randomly arranged linked list of pointers which reliably cause 
/// loads to miss in the cache.
///
/// Returns the difference (number of APERF cycles elapsed) in RAX.
///
/// ## Register use
/// - RCX cannot be overwritten (RDPRU depends on RCX=1)
/// - RDI/RSI cannot be overwritten (pointers for high-latency loads)
/// - R13 cannot be overwritten (loop counter)
/// - R14 cannot be overwritten (holds the initial value from APERF)
/// - R15 is a pointer for use by measured instructions
///
#[macro_export]
macro_rules! emit_hwong_gadget_test {
    ($tgt_ptr1:ident, $tgt_ptr2:ident, $free_ptr:ident, 
     $loop_iters:expr, $outer_unroll:expr, $inner_unroll:expr,
     body_a($($body_a:tt)*), body_b($($body_b:tt)*)
    ) => { {
        let mut asm = Assembler::<X64Relocation>::new().unwrap();
        emit_push_abi!(asm);
        dynasm!(asm
            // We have to keep 1 in RCX to read APERF with RDPRU
            // (this means measured code must not use RCX)
            ; mov       rcx, 1
            ; mov       rdi, QWORD $tgt_ptr1 as _
            ; mov       rsi, QWORD $tgt_ptr2 as _
            ; xor       r14, r14
            ; mov       r15, QWORD $free_ptr as _
        );

        // Take the first measurement.
        emit_rdpru_rdx!(asm,
            ; sub       r14, rdx
        );

        emit_loop_reg!(asm, r13, $loop_iters, {
            for _ in 0 ..$outer_unroll {
                dynasm!(asm ; mov   rdi, [rdi]);
                for _ in 0..$inner_unroll { dynasm!(asm $($body_a)*); }
                dynasm!(asm ; mov   rsi, [rsi]);
                for _ in 0..$inner_unroll { dynasm!(asm $($body_b)*); }
            }
        });

        // Take the second measurement.
        emit_rdpru_rdx!(asm,
            ; add       r14, rdx
            ; mov       rax, r14
        );

        emit_pop_abi_ret!(asm);
        asm.finalize().unwrap()
    } }
}

pub fn emit_rdpmc_test(ctr: u32) -> ExecutableBuffer {
    assert!(ctr < 6);
    let mut asm = Assembler::<X64Relocation>::new().unwrap();
    emit_push_abi!(asm);
    dynasm!(asm
        ; mov       ecx, ctr as _
        ; .bytes    RDPMC
        ; lfence
        ; shl       rdx, 32
        ; or        rdx, rax
        ; sub       r14, rdx

        ; .bytes    RDPMC
        ; lfence
        ; shl       rdx, 32
        ; or        rdx, rax
        ; add       r14, rdx
        ; mov       rax, r14
    );
    emit_pop_abi_ret!(asm);
    asm.finalize().unwrap()
}

