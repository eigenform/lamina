
use dynasmrt::{ ExecutableBuffer, AssemblyOffset };
use iced_x86::{ 
    Decoder, DecoderOptions, Instruction, Formatter, IntelFormatter 
};

/// Pin the current process to a particular core.
pub fn pin_to_core(core_id: usize) {
    let mut cpuset = nix::sched::CpuSet::new();
    let this_pid = nix::unistd::Pid::from_raw(0);
    cpuset.set(core_id).unwrap();
    nix::sched::sched_setaffinity(this_pid, &cpuset).unwrap();
}

/// Print the disassembly for a particular [ExecutableBuffer].
pub fn disas(buf: &ExecutableBuffer) {
    let ptr: *const u8 = buf.ptr(AssemblyOffset(0));
    let addr: u64   = ptr as u64;
    let buf: &[u8]  = unsafe { std::slice::from_raw_parts(ptr, buf.len()) };

    let mut decoder = Decoder::with_ip(64, buf, addr, DecoderOptions::NONE);
    let mut formatter = IntelFormatter::new();
    formatter.options_mut().set_digit_separator("_");
    formatter.options_mut().first_operand_char_index();
    let mut output = String::new();
    let mut instr  = Instruction::default();

    while decoder.can_decode() {
        decoder.decode_out(&mut instr);
        output.clear();
        formatter.format(&instr, &mut output);

        let start_idx = (instr.ip() & 0xfff) as usize;
        let instr_bytes = &buf[start_idx..start_idx + instr.len()];
        let mut bytestr = String::new();
        for b in instr_bytes.iter() {
            bytestr.push_str(&format!("{:02x}", b));
        }
        println!("{:016x}: {:32} {}", instr.ip(), bytestr, output);
    }
}

/// XorShift64* PRNG implementation.
///
/// # Safety
/// This is not designed to be safe (nor sound): the quality of randomness 
/// doesn't matter much for our purposes here. Do **not** use this elsewhere.
pub struct Xorshift64 { 
    pub val: usize 
}
impl Xorshift64 {

    /// Create a new PRNG seeded with the time-stamp counter.
    pub fn new() -> Self {
        Self { val: unsafe { core::arch::x86_64::_rdtsc() as usize } }
    }
    /// Update the state of the PRNG and return the next value.
    pub fn next(&mut self) -> usize {
        let mut next = self.val;
        next ^= next >> 12;
        next ^= next << 25;
        next ^= next >> 27;
        next  = next.wrapping_mul(0x2545f4914f6cdd1d);
        self.val = next;
        next
    }
}


