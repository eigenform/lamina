//! PMC event definitions (for Zen 2).

/// Indicates the primitive type of a counter.
pub enum CounterType {
    /// A number of clock cycles.
    ClockCycle,

    /// A number of retired instructions.
    RetiredInstruction,
    /// A number of retired ops/micro-ops/complex-ops.
    RetiredOp,
    /// A number of dispatched ops/micro-ops/complex-ops.
    DispatchedOp,

    Undefined,
}

/// [Event] is an 12-bit unsigned integer in the `PERF_CTL` MSRs.
/// Each event may also be qualified by some unit mask (an 8-bit unsigned 
/// integer) which further indicates which sub-events should be counted.
///
/// This code is mainly being used on a Ryzen 7 3950X; all events here are 
/// from "PPR for AMD Family 17h Model 71h B0" (56176 Rev 3.06 - Jul 17, 2019) 
/// unless noted otherwise. 
///
#[derive(Clone, Copy, Debug)]
#[allow(non_camel_case_types)]
pub enum Event {

    Undefined(u16, u8),

    /// PMCx027 - "Retired CPUID Instructions"
    LsRetCpuid(u8),
    
    /// PMCx029 - "Load/Store Dispatch"
    LsDispatch(u8),
    LdStDispatch,
    StDispatch,
    LdDispatch,

    /// PMCx02b - "SMIs Received"
    LsSmiRx(u8),

    /// PMCx02c - "Interrupts Taken"
    LsIntTaken(u8),

    /// PMCx02d - "Time Stamp Counter Reads (speculative)"
    LsRdTsc(u8),

    /// PMCx04b - "Software Prefetch Instructions Dispatched (speculative)"
    LsPrefInstrDisp(u8),

    /// PMCx076 - "Cycles Not In Halt"
    LsNotHaltedCyc(u8),

    /// PMCx0aa - "Source of Op Dispatched From Decoder"
    ///
    /// NOTE: This name is from "PPR Vol 1 for AMD Family 19h Model 01h B1"
    /// (55898 Rev 0.50 - May 27, 2021).
    DeSrcOpDisp(u8),
    OpCacheDispatched,
    DecoderDispatched,

    /// PMCx0ab - "Types of Ops Dispatched From Decoder"
    ///
    /// NOTE: From "PPR for AMD Family 19h Model 51h A1" 
    /// (56569-A1 Rev 3.03 - Sep 21, 2021)
    ///
    /// ## Unit Mask
    /// 0x80 - 0=IBS count, 1=retire count
    /// 0x08 - Any integer dispatch
    /// 0x04 - Any FP dispatch
    DeDisOpsFromDecoder(u8),

    /// PMC0x0ae - "Dispatch Resource Stalls 1"
    DeDisDispatchTokenStalls1(u8),
    IntSchedulerMiscRsrcStall,
    StoreQueueRsrcStall,
    LoadQueueRsrcStall,
    IntPhyRegFileRsrcStall,

    /// PMC0x0af - "Dispatch Resource Stalls 0"
    DeDisDispatchTokenStalls0(u8),
    ScAguDispatchStall,
    RetireTokenStall,
    AGSQTokenStall,
    ALUTokenStall,
    ALSQ3_0_TokenStall,
    ALSQ2RsrcStall,
    ALSQ1RsrcStall,

    /// PMCx0c0 - "Retired Instructions"
    ExRetInstr(u8),

    /// PMCx0c1 - "Retired Ops"
    ExRetCops(u8),

}
impl Event {

    /// Convert an [Event] and unit mask into a pair of integers.
    pub fn convert(&self) -> (u16, u8) {
        use Event::*;
        match self {
            Undefined(e, m)               => (*e & 0xfff, *m),

            LsRetCpuid(m)                 => (0x0027, *m),
            LsDispatch(m)                 => (0x0029, *m),
            LdStDispatch                  => (0x0029, 0x04),
            StDispatch                    => (0x0029, 0x02),
            LdDispatch                    => (0x0029, 0x01),
            LsSmiRx(m)                    => (0x002b, *m),
            LsIntTaken(m)                 => (0x002c, *m),
            LsRdTsc(m)                    => (0x002d, *m),
            LsPrefInstrDisp(m)            => (0x004b, *m),
            LsNotHaltedCyc(m)             => (0x0076, *m),
            DeSrcOpDisp(m)                => (0x00aa, *m),
            OpCacheDispatched             => (0x00aa, 0x02),
            DecoderDispatched             => (0x00aa, 0x01),
            DeDisOpsFromDecoder(m)        => (0x00ab, *m),
            DeDisDispatchTokenStalls1(m)  => (0x00ae, *m),
            IntSchedulerMiscRsrcStall     => (0x00ae, 0x08),
            StoreQueueRsrcStall           => (0x00ae, 0x04),
            LoadQueueRsrcStall            => (0x00ae, 0x02),
            IntPhyRegFileRsrcStall        => (0x00ae, 0x01),
            DeDisDispatchTokenStalls0(m)  => (0x00af, *m),
            ScAguDispatchStall            => (0x00af, 0x40),
            RetireTokenStall              => (0x00af, 0x20),
            AGSQTokenStall                => (0x00af, 0x10),
            ALUTokenStall                 => (0x00af, 0x08),
            ALSQ3_0_TokenStall            => (0x00af, 0x03),
            ALSQ2RsrcStall                => (0x00af, 0x02),
            ALSQ1RsrcStall                => (0x00af, 0x01),
            ExRetInstr(m)                 => (0x00c0, *m),
            ExRetCops(m)                  => (0x00c1, *m),
        }
    }
}


