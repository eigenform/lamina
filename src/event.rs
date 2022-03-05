//! PMC event definitions (for Zen 2).


/// [Event] is an 12-bit unsigned integer in the `PERF_CTL` MSRs.
/// Each event may also be qualified by some unit mask (an 8-bit unsigned 
/// integer) which further indicates which sub-events should be counted.
///
/// This code is mainly being used on a Ryzen 7 3950X; all events here are 
/// from "PPR for AMD Family 17h Model 71h B0" (56176 Rev 3.06 - Jul 17, 2019) 
/// unless noted otherwise. 
///
/// NOTE: I don't have a nice interface for specifying unit masks, so I'm just 
/// going to leave documentation in these comments for now?
#[derive(Clone, Copy, Debug)]
pub enum Event {

    /// PMCx027 - Retired CPUID Instructions
    LsRetCpuid(u8),
    
    /// PMCx029 - Load/Store Dispatch
    ///
    /// ## Unit Mask
    /// 0x01 - dispatched loads
    /// 0x02 - dispatched stores
    /// 0x04 - dispatched load+store ops
    LsDispatch(u8),

    /// PMCx02d - Time Stamp Counter Reads (speculative)
    LsRdTsc(u8),

    /// PMCx04b - Software Prefetch Instructions Dispatched (speculative)
    ///
    /// ## Unit Mask
    /// 0x01 - PrefetchT{0,1,2}
    /// 0x02 - PrefetchW
    /// 0x04 - PrefetchNTA
    LsPrefInstrDisp(u8),

    /// PMCx0aa - Source of Op Dispatched From Decoder
    ///
    /// NOTE: This is from "PPR Vol 1 for AMD Family 19h Model 01h B1"
    /// (55898 Rev 0.50 - May 27, 2021).
    ///
    /// ## Unit Mask
    /// 0x01 - fetched from IC and dispatched
    /// 0x02 - fetched from OC and dispatched
    DeSrcOpDisp(u8),

    /// PMCx0ab - Types of Ops Dispatched From Decoder
    ///
    /// NOTE: This is from 19h PPRs.
    ///
    /// ## Unit Mask
    /// 0x04 - all FP ops
    /// 0x08 - all integer ops
    /// 0x80 - 0=IBS count, 1=retire count
    DeDisOpsFromDecoder(u8),

    /// PMCx0c0 - Retired Instructions
    ExRetInstr(u8),
    /// PMCx0c1 - Retired Ops (uops/complex ops)
    ExRetCops(u8),
    /// PMCx0c3 - Retired Branch Instructions Mispredicted
    ExRetBrnMisp(u8),
}
impl Event {
    /// Convert an [Event] and unit mask into a pair of integers.
    pub fn convert(&self) -> (u16, u8) {
        match self {
            Self::LsRetCpuid(m)           => (0x0027, *m),
            Self::LsDispatch(m)           => (0x0029, *m),
            Self::LsRdTsc(m)              => (0x002d, *m),
            Self::LsPrefInstrDisp(m)      => (0x004b, *m),
            Self::DeSrcOpDisp(m)          => (0x00aa, *m),
            Self::DeDisOpsFromDecoder(m)  => (0x00ab, *m),
            Self::ExRetInstr(m)           => (0x00c0, *m),
            Self::ExRetCops(m)            => (0x00c1, *m),
            Self::ExRetBrnMisp(m)         => (0x00c3, *m),
        }
    }
}


