//! Definitions and wrapper types for dealing with MSRs associated with PMCs.
//!
//! The [PerfCtl] type is used to create a valid value for a particular 
//! `PERF_CTL` MSR. [PerfCtlDescriptor] represents a set of all six `PERF_CTL`
//! MSR values at some instant.

use crate::event::*;

/// Wrapper type for the set of all `PERF_CTL` bits.
#[derive(Clone, Copy, Debug)]
pub struct PerfCtlDescriptor {
    pub events: [Option<Event>; 6],
    pub ctl: [Option<PerfCtl>; 6],
}
impl PerfCtlDescriptor {
    /// Create a new, empty [PerfCtlDescriptor].
    pub fn new() -> Self { 
        Self {
            events: [None; 6],
            ctl: [None; 6],
        }
    }
    /// Clear all entries.
    pub fn clear_all(&mut self) { 
        self.events = [None; 6]; 
        self.ctl = [None; 6]; 
    }
    /// Return the 64-bit `PERF_CTL` value for this entry.
    pub fn get(&self, idx: usize) -> u64 {
        if let Some(ctl) = self.ctl[idx] { ctl.0 as u64 } else { 0 }
    }
    /// Clear (zero out) a particular entry.
    pub fn clear(&mut self, idx: usize) {
        assert!(idx < 6);
        self.ctl[idx] = None;
        self.events[idx] = None;
    }
    /// Set a particular entry.
    pub fn set(mut self, idx: usize, e: Event) -> Self {
        assert!(idx < 6);

        if e == Event::Merge {
            // NOTE: Eventually I'll test this to see what happens
            if (idx & 1) == 0 {
                panic!("Merge behavior undefined for even-numbered counters");
            }
            self.ctl[idx] = Some(PerfCtl::new_merge(true));
        } else {
            self.ctl[idx] = Some(PerfCtl::new(e, true));
        }
        self.events[idx] = Some(e);
        self
    }
}

/// Representing the host/guest field in a [PerfCtl] register.
#[derive(Clone, Copy, Debug)]
pub enum HostGuestBits {
    All       = 0b00,
    SVMEGuest = 0b01,
    SVMEHost  = 0b10,
    SVMEAll   = 0b11,
}
/// Representing the OS/user field in a [PerfCtl] register.
#[derive(Clone, Copy, Debug)]
pub enum OSUserBits {
    None = 0b00,
    User = 0b01,
    OS   = 0b10,
    All  = 0b11,
}

/// Wrapper type for the value of a `PERF_CTL` MSR.
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
pub struct PerfCtl(pub usize);
impl PerfCtl {

    // Quick visual reference for different fields in `PERF_CTL`.
    //
    // NOTE: Events are 12-bits, but honestly, I wouldn't be suprised if those 
    // high reserved bits are the high 4-bits of a *16-bit* space of events.
    // You can experiment with reserved bits when you're feeling brave enough.
    // 
    //   41                                                    0
    //    v                                                    v
    //    hh rrrr eeee cccccccc v n r i r g oo mmmmmmmm eeeeeeee
    //  0b00_0000_0000_00000000_0_0_0_0_0_0_00_00000000_00000000

    // "Count only host/guest events"
    pub const HOSTGUEST_MASK: usize = {
        0b11_0000_0000_00000000_0_0_0_0_0_0_00_00000000_00000000
    };

    // "Performance event select [11:8]"
    pub const EVTSEL_HI_MASK: usize = {
        0b00_0000_1111_00000000_0_0_0_0_0_0_00_00000000_00000000
    };

    // "Controls the number of events counted per clock cycle"
    pub const CNTMASK_MASK: usize = {
        0b00_0000_0000_11111111_0_0_0_0_0_0_00_00000000_00000000
    };

    // "Invert counter mask"
    pub const INV_MASK: usize = {
        0b00_0000_0000_00000000_1_0_0_0_0_0_00_00000000_00000000
    };

    // "Enable performance counter"
    pub const EN_MASK: usize = {
        0b00_0000_0000_00000000_0_1_0_0_0_0_00_00000000_00000000
    };

    // "Enable APIC interrupt"
    pub const INT_MASK: usize = {
        0b00_0000_0000_00000000_0_0_0_1_0_0_00_00000000_00000000
    };

    // "Edge detect"
    pub const EDGE_MASK: usize = {
        0b00_0000_0000_00000000_0_0_0_0_0_1_00_00000000_00000000
    }; 

    // "OS and user mode"
    pub const OSUSER_MASK: usize = {
        0b00_0000_0000_00000000_0_0_0_0_0_0_11_00000000_00000000
    };

    // "Event qualification"
    pub const UNITMASK_MASK: usize = {
        0b00_0000_0000_00000000_0_0_0_0_0_0_00_11111111_00000000
    };

    // "Event select [7:0]"
    pub const EVTSEL_LO_MASK: usize = {
        0b00_0000_0000_00000000_0_0_0_0_0_0_00_00000000_11111111
    };

    pub fn hostguest(&self) -> usize { (self.0 & Self::HOSTGUEST_MASK) >> 40 }
    pub fn event_select(&self) -> usize { 
          (self.0 & Self::EVTSEL_HI_MASK) >> 24 
        | (self.0 & Self::EVTSEL_LO_MASK)
    }
    pub fn count_mask(&self) -> usize { (self.0 & Self::CNTMASK_MASK) >> 24 }
    pub fn inv(&self) -> bool { (self.0 & Self::INV_MASK) != 0 }
    pub fn en(&self) -> bool { (self.0 & Self::EN_MASK) != 0 }
    pub fn int(&self) -> bool { (self.0 & Self::INT_MASK) != 0 }
    pub fn edge(&self) -> bool { (self.0 & Self::EDGE_MASK) != 0 }
    pub fn osuser(&self) -> usize { (self.0 & Self::OSUSER_MASK) >> 16 }
    pub fn unit_mask(&self) -> usize { (self.0 & Self::UNITMASK_MASK) >> 8 }
}

impl PerfCtl {

    /// Create a new `PERF_CTL` value.
    ///
    /// ## Defaults
    /// These are essentially commands to the PMC interface. 
    /// The default settings for these are as follows:
    ///
    /// - Only count events from userspace
    /// - Do *not* fire interrupts 
    /// - Don't worry about SVM-related bits or counter-overflow behavior
    ///
    pub fn new(event: Event, en: bool) -> Self {
        let mut res = Self(0);
        let e = event.convert();
        res.set_hostguest(HostGuestBits::All);
        res.set_event_select(e.0);
        res.set_count_mask(0);
        res.set_inv(false);
        res.set_en(en);
        res.set_int(false);
        res.set_edge(false);
        res.set_osuser(OSUserBits::User);
        res.set_unit_mask(e.1);
        res
    }

    /// Create a new `PERF_CTL` value for a merge event.
    /// All bits are unset except for 'en' and the event select.
    pub fn new_merge(en: bool) -> Self {
        let mut res = Self(0);
        res.set_event_select(Event::convert(&Event::Merge).0);
        res.set_en(en);
        res
    }

    pub fn clear(&mut self) { 
        self.0 = 0;
    }
    pub fn set_hostguest(&mut self, x: HostGuestBits) {
        self.0 = (self.0 & !Self::HOSTGUEST_MASK) | (x as usize) << 40;
    }
    pub fn set_event_select(&mut self, x: u16) {
        let x = x as usize;
        self.0 = (self.0 & !(Self::EVTSEL_HI_MASK|Self::EVTSEL_LO_MASK)) 
            | (x & 0b1111_00000000) << 24 | (x & 0b0000_11111111);
    }
    pub fn set_count_mask(&mut self, x: usize) {
        self.0 = (self.0 & !Self::CNTMASK_MASK) | (x & 0b11111111) << 24
    }
    pub fn set_inv(&mut self, x: bool) {
        self.0 = (self.0 & !Self::INV_MASK) | (x as usize) << 23
    }
    pub fn set_en(&mut self, x: bool) {
        self.0 = (self.0 & !Self::EN_MASK) | (x as usize) << 22
    }
    pub fn set_int(&mut self, x: bool) {
        self.0 = (self.0 & !Self::INT_MASK) | (x as usize) << 20
    }
    pub fn set_edge(&mut self, x: bool) {
        self.0 = (self.0 & !Self::EDGE_MASK) | (x as usize) << 18
    }
    pub fn set_osuser(&mut self, x: OSUserBits) {
        self.0 = (self.0 & !Self::OSUSER_MASK) | (x as usize) << 16
    }
    pub fn set_unit_mask(&mut self, x: u8) {
        let x = x as usize;
        self.0 = (self.0 & !Self::UNITMASK_MASK) | (x & 0b11111111) << 8
    }
}

