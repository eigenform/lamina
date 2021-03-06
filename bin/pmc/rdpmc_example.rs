//! Playing with PMCs and measuring speculative events.

use lamina::*;
use lamina::ctx::PMCContext;
use lamina::pmc::PerfCtlDescriptor;
use lamina::event::Event;


fn main() -> Result<(), &'static str> {
    // The kernel module always instruments PMCs on core 0
    lamina::util::pin_to_core(0);

    // Context for interactions with the kernel module
    let mut ctx = PMCContext::new()?;
    let pmc = PerfCtlDescriptor::new()
        .set(0, Event::ExRetCops(0x00))
        .set(1, Event::DeSrcOpDisp(0x03))
        .set(2, Event::ExRetInstr(0x00))
        .set(3, Event::LsNotHaltedCyc(0x00))
        .set(4, Event::LsIntTaken(0x00))
        .set(5, Event::LsSmiRx(0x00));
    ctx.write(&pmc)?;

    let code = emit_rdpmc_test_all!();
    let mut test = PMCTest::new("floor", &code, &pmc);
    test.run_iter(0x1000);
    test.print();
    println!("");

    let code = emit_rdpmc_test_all!(
        ; nop
        ; nop
        ; nop
        ; nop
    );
    let mut test = PMCTest::new("4 nops", &code, &pmc);
    test.run_iter(0x1000);
    test.print();



    Ok(())
}
