//! Playing with PMCs and measuring speculative events.

use lamina::PMCContext;
use lamina::pmc::{ PerfCtl, PerfCtlDescriptor };
use lamina::event::Event;
use lamina::x86::*;
use lamina::*;

use dynasmrt::{
    dynasm, DynasmApi, DynasmLabelApi, 
    Assembler, AssemblyOffset, ExecutableBuffer, 
    x64::X64Relocation
};


fn main() -> Result<(), &'static str> {
    // The kernel module always instruments PMCs on core 0
    lamina::util::pin_to_core(0);

    // Context for interactions with the kernel module
    let mut ctx = PMCContext::new()?;
    let mut pmc = PerfCtlDescriptor::new()
        .set(0, PerfCtl::new(Event::LsRdTsc(0x00), true));
    ctx.write(&pmc);

    // Scratch pointer for emitted code
    let mut scratch = Box::new([0u8; 64]);
    let scratch_ptr = scratch.as_ptr();

    // Get the floor/number of ambient events for emit_rdpmc_test!() gadget.
    // You should see no LsRdTsc events.
    let test = emit_rdpmc_test!(0, );
    let func = TestFunc::new(&test);
    let res = func.run_iter(0x2000);
    println!("floor      {:?}", res);

    // Run a test where RDTSC is executed speculatively.
    // You should see at most 1 LsRdTsc event. 
    //
    let test = emit_rdpmc_test!(0, 
        ; mov rdi, QWORD scratch_ptr as _
        ; call ->func

        ; rdtsc
        ; jmp ->end

        ; ->func:
        ; lea rax, [->end]
        ; xchg [rsp], rax
        ; sfence
        ; ret

        ; ->end:
        ; mov [rdi], rdx
        ; mfence
        ; nop
    );
    let func = TestFunc::new(&test);
    let res = func.run_iter(0x2000);
    println!("spec_rdtsc {:?}", res);

    // Run a test where a #UD stops speculation before reaching RDTSC.
    // You should see no LsRdTsc events.
    //
    // See "Speculation Behavior in AMD Micro-Architectures":
    //
    // > Some faults are detected as the processor is decoding the instruction. 
    // > These include instruction breakpoints (#DB), invalid opcode (#UD), 
    // > instruction page fault (#PF) and device not available (#NM). 
    // > These fault types do not allow *dispatch* of the current instruction
    // > on which the fault is detected or any younger instruction.
    //
    let test = emit_rdpmc_test!(0,
        ; mov rdi, QWORD scratch_ptr as _
        ; call ->func

        ; ud2
        ; rdtsc
        ; jmp ->end

        ; ->func:
        ; lea rax, [->end]
        ; xchg [rsp], rax
        ; sfence
        ; ret

        ; ->end:
        ; mov [rdi], rdx
        ; mfence
        ; nop
    );
    let func = TestFunc::new(&test);
    let res  = func.run_iter(0x2000);
    println!("spec_#ud   {:?}", res);

    // Run a test where #GP stops speculation before reaching RDTSC.
    // You should see no LsRdTsc events.
    //
    // (Like #UD, we'd expect that #GP should also prevent any speculative 
    // dispatch of younger instructions)
    //
    let test = emit_rdpmc_test!(0,
        ; mov rdi, QWORD scratch_ptr as _
        ; call ->func

        ; mov ecx, 0x10
        ; rdmsr
        ; rdtsc
        ; jmp ->end

        ; ->func:
        ; lea rax, [->end]
        ; xchg [rsp], rax
        ; sfence
        ; ret

        ; ->end:
        ; mov [rdi], rdx
        ; mfence
        ; nop
    );
    let func = TestFunc::new(&test);
    let res  = func.run_iter(0x2000);
    println!("spec_#gp   {:?}", res);

    Ok(())
}
