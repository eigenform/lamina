#[allow(unused_macros)]

pub mod codegen;
pub mod util;
pub mod chase;
pub mod x86;
pub mod pmc;

use std::convert::TryInto;
use dynasmrt::{ ExecutableBuffer, AssemblyOffset };

/// Path to the character device exposed by the kernel module.
pub const LAMINA_CHARDEV: &str = "/dev/lamina";
pub const CMD_WRITECTL: usize = 0x0000_1000;

/// Set of PERF_CTL values to-be-written by the kernel module.
#[repr(C)]
pub struct LaminaMsg { ctl: [u64; 6] }
impl LaminaMsg {
    pub fn new(desc: pmc::PerfCtlDescriptor) -> Self { 
        let mut res = LaminaMsg { ctl: [0; 6] };
        res.ctl[0] = desc.get(0);
        res.ctl[1] = desc.get(1);
        res.ctl[2] = desc.get(2);
        res.ctl[3] = desc.get(3);
        res.ctl[4] = desc.get(4);
        res.ctl[5] = desc.get(5);
        res
    }
}

nix::ioctl_write_ptr_bad! {
    /// Send a set of PERF_CTL values to the kernel module.
    lamina_writectl, CMD_WRITECTL, LaminaMsg
}

/// Try to get a file descriptor for the lamina character device.
pub fn lamina_open() -> Result<i32, &'static str> {
    use nix::sys::stat::Mode;
    use nix::fcntl::{ open, OFlag };
    use nix::errno::Errno;
    match open(LAMINA_CHARDEV, OFlag::O_RDWR, Mode::S_IRWXU) {
        Ok(fd) => Ok(fd),
        Err(e) => match e {
            Errno::ENOENT => Err("Kernel module not loaded?"),
            Errno::EACCES => Err("Permission denied?"),
            _ => panic!("unhandled error {}", e),
        },
    }
}

/// Close the file descriptor bound to the lamina character device.
pub fn lamina_close(fd: i32) {
    use nix::unistd::close;
    match close(fd) {
        Ok(_) => {},
        Err(e) => panic!("{}", e),
    }
}



/// Call into a block of emitted code.
pub fn run_test(buf: &ExecutableBuffer) -> usize {
    let ptr: *const u8 = buf.ptr(AssemblyOffset(0));
    let line_ptr = ptr as *const [u8; 64];
    unsafe {
        let func: extern "C" fn() -> usize = std::mem::transmute(ptr);

        // Flush all emitted code from the caches
        for idx in 0..(buf.len() / 64) + 1 {
            core::arch::x86_64::_mm_clflush(
                line_ptr.offset(idx.try_into().unwrap()) as *const u8
            )
        }
        core::arch::x86_64::_mm_mfence();
        core::arch::x86_64::_mm_lfence();

        func()
    }
}


