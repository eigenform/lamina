
use lamina::PMCContext;
use lamina::pmc::{ PerfCtl, PerfCtlDescriptor };
use lamina::event::Event;

//use dynasmrt::{
//    dynasm, DynasmApi, DynasmLabelApi, 
//    Assembler, AssemblyOffset, ExecutableBuffer, 
//    x64::X64Relocation
//};

fn main() -> Result<(), &'static str> {
    // The kernel module always instruments PMCs on core 0
    lamina::util::pin_to_core(0);

    // Context for interactions with the kernel module
    let mut ctx = PMCContext::new()?;

    // Create a new set of PMCs
    let mut pmc = PerfCtlDescriptor::new()
        .set(0, PerfCtl::new(Event::ExRetInstr(0), true));

    // Write/enable PMCs
    ctx.write(&pmc);

    // Emit and run some test that uses RDPMC
    let test = lamina::codegen::emit_rdpmc_test(0);
    let ptr: *const u8 = test.ptr(dynasmrt::AssemblyOffset(0));
    let mut v = vec![0; 0x1000];
    for i in 0..0x1000 {
        let res = unsafe {
            let func: extern "C" fn() -> usize = std::mem::transmute(ptr);
            func()
        };
        v[i] = res;
    }

    println!("min={} max={} avg={:0.2}", 
        v.iter().min().unwrap(), v.iter().max().unwrap(),
        v.iter().sum::<usize>() as f32 / v.len() as f32
    );

    Ok(())
}
