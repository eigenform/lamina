//! PMC event definitions (for Zen 2).

/// Some property that characterizes an event.
pub enum EventProperty {
    Retired,
    Dispatched,
}

/// Indicates the primitive type of a counter.
pub enum CounterUnit {
    ClockCycle,
    Instruction(EventProperty),
    Op(EventProperty),
    UndefinedUnit,
}
impl CounterUnit {
    pub fn to_str(&self) -> &'static str {
        use CounterUnit::*;
        use EventProperty::*;
        match self {
            ClockCycle => "Clock cycles",
            Instruction(p) => match p {
                Retired => "Retired instructions",
                Dispatched => "Dispatched instructions",
            },
            Op(p) => match p {
                Retired => "Retired ops",
                Dispatched => "Dispatched ops",
            }
            UndefinedUnit => "Undefined",
        }
    }
}

/// A description of an event.
pub struct EventDesc {
    pub desc: &'static str,
    pub unit: CounterUnit
}


/// [Event] is an 12-bit unsigned integer in the `PERF_CTL` MSRs.
/// Each event may also be qualified by some unit mask (an 8-bit unsigned 
/// integer) which further indicates which sub-events should be counted.
///
/// This code is mainly being used on a Ryzen 9 3950X; all events here are 
/// from "PPR for AMD Family 17h Model 71h B0" (56176 Rev 3.06 - Jul 17, 2019) 
/// unless noted otherwise. 
///
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[allow(non_camel_case_types)]
pub enum Event {

    Undefined(u16, u8),
    Merge,

    /// PMCx025 - "Retired Lock Instructions"
    LsLocks(u8),
    SpecLockHiSpec,
    SpecLockLoSpec,
    NonSpecLock,
    BusLock,

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
    /// PMCx035 - "Number of Store-to-Load Forwarding hits"
    LsSTLF(u8),
    /// PMCx04b - "Software Prefetch Instructions Dispatched (speculative)"
    LsPrefInstrDisp(u8),
    /// PMCx076 - "Cycles Not In Halt"
    LsNotHaltedCyc(u8),

    /// PMCx08a - "L1 Branch Prediction Overrides Existing Prediction" (speculative)
    BpL1BTBCorrect(u8),
    /// PMCx08b - "L2 Branch Prediction Overrides Existing Prediction" (speculative)
    BpL2BTBCorrect(u8),
    /// PMCx08e - "Dynamic Indirect Predictions"
    BpDynIndPred(u8),
    /// PMCx091 - "Decoder Overrides Existing Branch Prediction"
    BpDeReDirect(u8),

    /// PMCx0aa - "Source of Op Dispatched From Decoder"
    ///
    /// NOTE: This name is from "PPR Vol 1 for AMD Family 19h Model 01h B1"
    /// (55898 Rev 0.50 - May 27, 2021). 
    ///
    /// Also, see errata #1287 in the 19h revision guide, which may be related. 
    /// This does not seem to count correctly, or the behavior is simply not 
    /// well-defined in the 17h PPRs. This seems to undercount when compared 
    /// to PMC0xab.
    ///
    DeSrcOpDisp(u8),
    OpCacheDispatched,
    DecoderDispatched,

    /// PMCx0ab - "Types of Ops Dispatched From Decoder"
    ///
    /// NOTE: From "PPR for AMD Family 19h Model 51h A1" 
    /// (56569-A1 Rev 3.03 - Sep 21, 2021)
    ///
    /// This seems like it also counts speculatively-dispatched ops.
    ///
    /// ## Unit Mask
    /// The PPR mentions that unit mask `0x08` selects dispatched integer 
    /// ops, and unit mask `0x04` selects dispatched floating-point ops.
    /// Masks `0x01` and `0x02` seem to further distinguish between different
    /// types of integer ops - may be related to "fastpath"/microcoded ops?
    ///
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
    /// PMCx0c2 - "Retired Branch Instructions"
    ExRetBrn(u8),
    /// PMCx0c3 - "Retired Branch Instructions Mispredicted"
    ExRetBrnMisp(u8),
    /// PMCx0c7 - "Retired Branch Resyncs"
    ExRetBrnResync(u8),
    /// PMCx0c9 - "Retired Near Returns Mispredicted"
    ExRetNearRetMispred(u8),
    /// PMCx0ca - "Retired Indirect Branch Instructions Mispredicted"
    ExRetBrnIndMisp(u8),

}

impl Event {
    /// Return a description of this event.
    pub fn desc(&self) -> EventDesc {
        use Event::*;
        use CounterUnit::*;
        use EventProperty::*;
        match self {
            LsPrefInstrDisp(_) => EventDesc { 
                desc: "Dispatched PREFETCH instructions (speculative)",
                unit: Instruction(Dispatched),
            },

            BpL1BTBCorrect(_) => EventDesc {
                desc: "Branch redirects from L1 BTB (speculative)",
                unit: UndefinedUnit,
            },
            BpL2BTBCorrect(_) => EventDesc {
                desc: "Branch redirects from L2 BTB (speculative)",
                unit: UndefinedUnit,
            },
            BpDynIndPred(_) => EventDesc {
                desc: "Dynamic indirect branch predictions",
                unit: UndefinedUnit,
            },
            BpDeReDirect(_) => EventDesc {
                desc: "Branch redirects from decoder",
                unit: UndefinedUnit,
            },

            DeDisOpsFromDecoder(_) => EventDesc {
                desc: "Dispatched ops from decoder (speculative)",
                unit: Op(Dispatched),
            },

            ExRetInstr(_)   => EventDesc { 
                desc: "Retired instructions",
                unit: Instruction(Retired),
            },
            ExRetCops(_)    => EventDesc {
                desc: "Retired ops",
                unit: Op(Retired),
            },
            ExRetBrn(_)     => EventDesc { 
                desc: "Retired branch instructions",
                unit: Instruction(Retired),
            },
            ExRetBrnMisp(_) => EventDesc {
                desc: "Retired branch instructions (mispredicted)",
                unit: Instruction(Retired),
            },

            ExRetNearRetMispred(_) => EventDesc {
                desc: "Retired near-return instructions (mispredicted)",
                unit: Instruction(Retired),
            },
            ExRetBrnIndMisp(_) => EventDesc {
                desc: "Retired indirect branch instructions (mispredicted)",
                unit: Instruction(Retired),
            },

            _ => EventDesc { 
                desc: "No description provided",
                unit: UndefinedUnit
            },
        }
    }

    /// Convert an [Event] and unit mask into a pair of integers.
    pub fn convert(&self) -> (u16, u8) {
        use Event::*;
        match self {
            Undefined(e, m)               => (*e & 0xfff, *m),
            Merge                         => (0xfff, 0x00),

            LsLocks(m)                    => (0x0025, *m),
            SpecLockHiSpec                => (0x0025, 0x08),
            SpecLockLoSpec                => (0x0025, 0x04),
            NonSpecLock                   => (0x0025, 0x02),
            BusLock                       => (0x0025, 0x01),

            LsRetCpuid(m)                 => (0x0027, *m),

            LsDispatch(m)                 => (0x0029, *m),
            LdStDispatch                  => (0x0029, 0x04),
            StDispatch                    => (0x0029, 0x02),
            LdDispatch                    => (0x0029, 0x01),

            LsSmiRx(m)                    => (0x002b, *m),
            LsIntTaken(m)                 => (0x002c, *m),
            LsRdTsc(m)                    => (0x002d, *m),
            LsSTLF(m)                     => (0x0035, *m),
            LsPrefInstrDisp(m)            => (0x004b, *m),
            LsNotHaltedCyc(m)             => (0x0076, *m),

            BpL1BTBCorrect(m)             => (0x008a, *m),
            BpL2BTBCorrect(m)             => (0x008b, *m),
            BpDynIndPred(m)               => (0x008e, *m),
            BpDeReDirect(m)               => (0x0091, *m),

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
            ExRetBrn(m)                   => (0x00c2, *m),
            ExRetBrnMisp(m)               => (0x00c3, *m),
            ExRetBrnResync(m)             => (0x00c7, *m),
            ExRetNearRetMispred(m)        => (0x00c9, *m),
            ExRetBrnIndMisp(m)            => (0x00ca, *m),
        }
    }
}


