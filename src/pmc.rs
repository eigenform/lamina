//! Definitions for performance-monitoring counters (PMCs).
//!
//! All of this data has either been recovered through experiment, or pulled
//! from public datasheets for Family 17h/Family 19h parts.
//!
//! ## Munging through AMD publications
//! If you're thinking about PMCs on Zen machines, you will probably become
//! confused if you're comparing between PPRs for different processors: 
//!
//! - There are weird and incomprehensible omissions of things
//! - There are changes to names of things without any rationale
//! - They leave a lot to be desired (many details are simply undefined)
//!
//! If you collect enough (see https://www.amd.com/en/support/tech-docs), you
//! can comb through lots of PDFs at once with one-liners (i.e. I've found
//! `pdftk` and `pdftotext` are pretty useful for combing through them).
//!

/// Wrapper type for the set of all PERF_CTL bits (for each counter).
pub struct PerfCtlDescriptor(pub [Option<usize>; 6]);
impl PerfCtlDescriptor {
    pub fn new() -> Self {
        Self([None; 6])
    }
    pub fn get(&self, idx: usize) -> u64 {
        if let Some(val) = self.0[idx] { val as u64 } else { 0 }
    }
    pub fn clear(&mut self, idx: usize) {
        assert!(idx < 6);
        self.0[idx] = None;
    }
    pub fn set(&mut self, idx: usize, x: PerfCtl) {
        assert!(idx < 6);
        self.0[idx] = Some(x.0);
    }
}

/// Wrapper type for a set of PERF_CTL bits.
#[repr(transparent)]
pub struct PerfCtl(pub usize);
impl PerfCtl {

//       41
//        v
//        hh rrrr eeee cccccccc v e r i r g oo mmmmmmmm eeeeeeee
//      0b00_0000_0000_00000000_0_0_0_0_0_0_00_00000000_00000000

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

    pub fn hostguest(&self) -> usize { 
        (self.0 & Self::HOSTGUEST_MASK) >> 40 
    }
    pub fn event_select(&self) -> usize { 
          (self.0 & Self::EVTSEL_HI_MASK) >> 24 
        | (self.0 & Self::EVTSEL_LO_MASK)
    }
    pub fn count_mask(&self) -> usize { 
        (self.0 & Self::CNTMASK_MASK) >> 24 
    }
    pub fn inv(&self) -> bool { 
        (self.0 & Self::INV_MASK) != 0 
    }
    pub fn en(&self) -> bool { 
        (self.0 & Self::EN_MASK) != 0 
    }
    pub fn int(&self) -> bool { 
        (self.0 & Self::INT_MASK) != 0 
    }
    pub fn edge(&self) -> bool { 
        (self.0 & Self::EDGE_MASK) != 0 
    }
    pub fn osuser(&self) -> usize { 
        (self.0 & Self::OSUSER_MASK) >> 16 
    }
    pub fn unit_mask(&self) -> usize { 
        (self.0 & Self::UNITMASK_MASK) >> 8 
    }
}

impl PerfCtl {
    pub fn new(event_mask: u16, count_mask: usize, unit_mask: u8,
               inv: bool, en: bool, int: bool, edge: bool) -> Self {
        let mut res = Self(0);
        res.set_hostguest(0b00);
        res.set_event_select(event_mask);
        res.set_count_mask(count_mask);
        res.set_inv(inv);
        res.set_en(en);
        res.set_int(int);
        res.set_edge(edge);
        res.set_osuser(0b11);
        res.set_unit_mask(unit_mask);
        res
    }
    pub fn clear(&mut self) { 
        self.0 = 0;
    }
    pub fn set_hostguest(&mut self, x: usize) {
        self.0 = (self.0 & !Self::HOSTGUEST_MASK) | (x & 0b11) << 40;
    }
    pub fn set_event_select(&mut self, x: u16) {
        let x = x as usize;
        self.0 = (self.0 & !(Self::EVTSEL_HI_MASK|Self::EVTSEL_LO_MASK)) 
            | (x & 0b1111_00000000) << 24
            | (x & 0b0000_11111111);
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
    pub fn set_osuser(&mut self, x: usize) {
        self.0 = (self.0 & !Self::OSUSER_MASK) | (x & 0b11) << 16
    }
    pub fn set_unit_mask(&mut self, x: u8) {
        let x = x as usize;
        self.0 = (self.0 & !Self::UNITMASK_MASK) | (x & 0b11111111) << 8
    }
}

#[cfg(test)]
mod test {
    use crate::pmc::*;
    #[test]
    fn test() {
        let mut x = PerfCtl(0);
        x.set_hostguest(0b00);
        x.set_event_select(0x76);
        x.set_en(true);
        x.set_int(true);
        x.set_osuser(0b11);
        println!("{:016x}", x.0);
    }
}


