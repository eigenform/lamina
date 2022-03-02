//! PMC event definitions (for Zen 2).
//!
//! [Event] is an 12-bit unsigned integer in the `PERF_CTL` MSRs.
//! Each event may also be qualified by some unit mask (an 8-bit unsigned 
//! integer) which further indicates which sub-events should be counted.
//!

#[derive(Clone, Copy, Debug)]
pub enum Event {
    LsRetCpuid(u8),
    LsDispatch(u8),
    LsRdTsc(u8),
    DeDisUopsFromDecoder(u8),
    ExRetInstr(u8),
    ExRetCops(u8),
}
impl Event {
    pub fn convert(&self) -> (u16, u8) {
        match self {
            Self::LsRetCpuid(m)           => (0x0027, *m),
            Self::LsDispatch(m)           => (0x0029, *m),
            Self::LsRdTsc(m)              => (0x002d, *m),
            Self::DeDisUopsFromDecoder(m) => (0x00aa, *m),
            Self::ExRetInstr(m)           => (0x00c0, *m),
            Self::ExRetCops(m)            => (0x00c1, *m),
        }
    }
}


