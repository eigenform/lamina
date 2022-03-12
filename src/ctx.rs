//! Conventions for managing interactions with the kernel module.

use crate::pmc;

type Err<T> = Result<T, &'static str>;

/// Kernel module FFI - a set of PERF_CTL values.
#[repr(C)]
pub struct LaminaMsg { 
    ctl: [u64; 6] 
}

nix::ioctl_write_ptr_bad! {
    /// Kernel module FFI - write a new set of PERF_CTL values.
    lamina_writectl, PMCContext::CMD_WRITECTL, LaminaMsg
}

/// Container for the current state of the PMCs.
///
/// ## Safety
/// We only intend for a single process to interact with the kernel module
/// via a *single* open file descriptor - this is *not enforced* anywhere, and
/// we're flying by only *convention*.
///
/// The file descriptor *must* be closed when a [PMCContext] is dropped
/// (even though the kernel will probably drop it for us when we die). You 
/// probably also want to set/write an empty [pmc::PerfCtlDescriptor] before 
/// closing the file descriptor (to explicitly stop the counters).
///
pub struct PMCContext {
    /// File descriptor for the character device.
    fd: i32,
    /// The most recent set of PERF_CTL values.
    desc: pmc::PerfCtlDescriptor,
}

impl PMCContext {

    /// Path to the character device exposed by the kernel module.
    pub const CHARDEV: &'static str = "/dev/lamina";

    /// `ioctl()` command for writing a new set of PMC events.
    pub const CMD_WRITECTL: usize = 0x0000_1000;

    /// Create a new context.
    pub fn new() -> Err<Self> {
        use nix::sys::stat::Mode;
        use nix::fcntl::{ open, OFlag };
        use nix::errno::Errno;

        let fd = match open(Self::CHARDEV, OFlag::O_RDWR, Mode::S_IRWXU) {
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

    /// Send the associated [pmc::PerfCtlDescriptor] to the kernel module.
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

    /// Clear (zero out) the [pmc::PerfCtlDescriptor] for this context.
    pub fn clear(&mut self) -> Err<()> {
        self.desc.clear_all();
        self.do_ioctl()
    }

    /// Write a new [pmc::PerfCtlDescriptor] for this context.
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
            Err(_) => {
                println!("[!] Couldn't close lamina file descriptor?");
            },
        }
    }
}


