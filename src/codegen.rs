//! Macros for generating different microbenchmarks at runtime.
//!
//! These are mostly all wrappers around the [dynasmrt::dynasm] macro, used 
//! for easily parameterizing different kinds of gadgets/idioms in assembly.
//!
//! ## Safety
//!
//! Because a lot of logic is wrapped up in macros (allowing you to insert
//! code into templates fairly-easily), many of the guarantees about safety
//! here are based only on convention. You may want to read some of this code
//! before you use it.
//!
//! ## Notes on Compatibility
//!
//! - This library was written and tested on a Ryzen 7 3950X machine.
//! - This library is only intended for 64-bit Linux machines.
//!
//! When calling into emitted code, we follow the SysV ABI. 
//! The behavior of some of the assembly here might depend on the way that
//! the Zen 2 microarchitecture is implemented. Any compatibility with other
//! machines is *not expected* and *not guaranteed*.
//!


/// Common prologue for emitted code. 
///
/// Pushes the SysV ABI callee-save registers onto the stack.
/// Clears all of the general-purpose registers (with the exception of RSP).
///
/// ## Safety
///
/// Right now, stack usage in emitted code is unsupported.
/// At some point, we can probably have some way to switch into a new stack.
///
#[macro_export]
macro_rules! emit_push_abi { ($asm:ident) => {
    dynasm!($asm
        ; .arch     x64
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
        ; mfence
        ; lfence
    );
}}

/// Common epilogue for emitted code.
/// Pops the SysV ABI callee-save registers from the stack.
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
        ; pop       rbp
        ; ret
        ; lfence
    );
}}

/// Emit a bare loop using some register and the JNE instruction.
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


/// Emit a test utilizing all six PMC registers to capture some result data.
///
/// ## Conventions
/// r15 is reserved for a pointer to the set of results.
/// r14, r13, r12, r11, r10, and r9 are reserved for RDPMC results.
///
#[macro_export]
macro_rules! emit_rdpmc_test_all {
    ($($body:tt)*) => { {
        let mut asm = Assembler::<X64Relocation>::new().unwrap();
        dynasm!(asm
            ; .arch     x64
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

            ; mov       r15, rdi
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
            //; xor       r15, r15
            ; mfence
            ; lfence
        );

        // Take some measurements
        dynasm!(asm
            ; mov rcx, 5 ; lfence ; rdpmc ; lfence ; sub r14, rax
            ; mov rcx, 4 ; lfence ; rdpmc ; lfence ; sub r13, rax
            ; mov rcx, 3 ; lfence ; rdpmc ; lfence ; sub r12, rax
            ; mov rcx, 2 ; lfence ; rdpmc ; lfence ; sub r11, rax
            ; mov rcx, 1 ; lfence ; rdpmc ; lfence ; sub r10, rax
            ; mov rcx, 0 ; lfence ; rdpmc ; lfence ; sub  r9, rax
        );

        // Do something.
        // At this point, RAX, RCX, RDX, and R9-R14 have been used.

        dynasm!(asm 
            $($body)*
        );

        // Take another set of measurements and compute the difference
        dynasm!(asm
            ; mov rcx, 0 ; lfence ; rdpmc ; lfence ; add  r9, rax
            ; mov rcx, 1 ; lfence ; rdpmc ; lfence ; add r10, rax
            ; mov rcx, 2 ; lfence ; rdpmc ; lfence ; add r11, rax
            ; mov rcx, 3 ; lfence ; rdpmc ; lfence ; add r12, rax
            ; mov rcx, 4 ; lfence ; rdpmc ; lfence ; add r13, rax
            ; mov rcx, 5 ; lfence ; rdpmc ; lfence ; add r14, rax
        );

        // Write the results back to memory
        dynasm!(asm
            ; mov [r15 + 0x00], r9
            ; mov [r15 + 0x08], r10
            ; mov [r15 + 0x10], r11
            ; mov [r15 + 0x18], r12
            ; mov [r15 + 0x20], r13
            ; mov [r15 + 0x28], r14
        );

        dynasm!(asm
            ; pop r15
            ; pop r14
            ; pop r13
            ; pop r12
            ; pop rsi
            ; pop rdi
            ; pop rbx
            ; pop rbp
            ; mfence
            ; ret
            ; lfence
        );
        asm.finalize().unwrap()
    } }
}


/// Emit a test utilizing a single counter to capture a single event.
///
/// Returns the difference (number of events counted) in RAX.
///
/// NOTE: This is unusable right now because [PMCTest] (and implicitly,
/// [PMCTestFn] assume that emitted code writes all six counters to some
/// array given by a pointer.
///
#[macro_export]
macro_rules! emit_rdpmc_test_single {
    ($ctr:expr, $($body:tt)*) => { {
        assert!($ctr < 6);
        let mut asm = Assembler::<X64Relocation>::new().unwrap();
        emit_push_abi!(asm);
        dynasm!(asm
            ; mov       ecx, $ctr as _
            ; lfence
            ; rdpmc
            ; lfence
            ; sub r15, rax

            $($body)*

            ; mov       ecx, $ctr as _
            ; lfence
            ; rdpmc
            ; lfence
            ; add r15, rax
            ; mov rax, r15
            ; mfence
        );
        emit_pop_abi_ret!(asm);
        asm.finalize().unwrap()
    } }
}


