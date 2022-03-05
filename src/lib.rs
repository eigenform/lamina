#[allow(unused_macros)]

pub mod codegen;
pub mod util;
pub mod chase;
pub mod x86;
pub mod pmc;
pub mod event;

use std::convert::TryInto;
use dynasmrt::{ ExecutableBuffer, AssemblyOffset };

/// Path to the character device exposed by the kernel module.
pub const LAMINA_CHARDEV: &str = "/dev/lamina";
pub const CMD_WRITECTL: usize = 0x0000_1000;

type Err<T> = Result<T, &'static str>;

/// Set of PERF_CTL values to-be-written by the kernel module.
#[repr(C)]
pub struct LaminaMsg { ctl: [u64; 6] }

nix::ioctl_write_ptr_bad! {
    /// Send a set of PERF_CTL values to the kernel module.
    lamina_writectl, CMD_WRITECTL, LaminaMsg
}

/// Container for the current state of the PMCs.
pub struct PMCContext {
    desc: pmc::PerfCtlDescriptor,
    fd: i32,
}
impl PMCContext {
    pub fn new() -> Err<Self> {
        use nix::sys::stat::Mode;
        use nix::fcntl::{ open, OFlag };
        use nix::errno::Errno;

        let fd = match open(LAMINA_CHARDEV, OFlag::O_RDWR, Mode::S_IRWXU) {
            Ok(fd) => {
                Ok(fd)
            },
            Err(e) => match e {
                Errno::ENOENT => Err("Kernel module not loaded?"),
                Errno::EACCES => Err("Permission denied?"),
                _ => panic!("unhandled error {}", e),
            },
        }?;

        Ok(Self { desc: pmc::PerfCtlDescriptor::new(), fd, })
    }

    fn do_ioctl(&mut self) -> Err<()> {
        let mut msg = LaminaMsg { ctl: [0; 6] };
        for (idx, val) in msg.ctl.iter_mut().enumerate() {
            *val = self.desc.get(idx);
        }

        unsafe {
            match lamina_writectl(self.fd, &msg as *const LaminaMsg) {
                Ok(res) => {
                    if res < 0 {
                        return Err("ioctl() returned non-zero");
                    }
                },
                Err(e) => {
                    println!("{}", e);
                    return Err("ioctl() unspecified error");
                }
            }
        }
        Ok(())
    }

    pub fn clear(&mut self) -> Err<()> {
        self.desc.clear_all();
        self.do_ioctl()
    }
    pub fn write(&mut self, d: &pmc::PerfCtlDescriptor) -> Err<()> {
        self.desc = *d;
        self.do_ioctl()
    }

}
impl std::ops::Drop for PMCContext {
    fn drop(&mut self) {
        use nix::unistd::close;
        self.clear().unwrap();
        match close(self.fd) {
            Ok(_) => {},
            Err(_) => println!("[!] Couldn't close lamina file descriptor?"),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct TestResults {
    pub min: usize,
    pub max: usize,
}
impl std::ops::Sub for TestResults {
    type Output = Self;
    fn sub(self, rhs: Self) -> Self::Output {
        TestResults {
            min: self.min - rhs.min,
            max: self.max - rhs.max
        }
    }
}

/// Wrapper around a function pointer for emitted code.
///
/// NOTE: You'll want to fix this up later to deal with multiple counters.
pub struct TestFunc{ 
    size: usize,
    ptr:  *const u8,
    func: extern "C" fn() -> usize,
}
impl TestFunc {
    pub fn new(buf: &ExecutableBuffer) -> Self {
        let ptr: *const u8 = buf.ptr(AssemblyOffset(0));
        unsafe {
            Self { 
                ptr,
                size: buf.len(),
                func: std::mem::transmute(ptr),
            }
        }
    }

    /// Run the emitted code once.
    pub fn run_once(&self) -> usize { (self.func)() }

    /// Run emitted code some number of times.
    pub fn run_iter(&self, iter: usize) -> TestResults {
        let mut res = vec![0; iter];
        for i in 0..iter { 
            crate::util::clflush(self.size, self.ptr as *const [u8; 64]);
            res[i] = (self.func)();
        }
        TestResults {
            min: *res.iter().min().unwrap(),
            max: *res.iter().max().unwrap(),
        }
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


