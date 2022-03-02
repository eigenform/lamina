
use crate::event::*;

/// Wrapper type for the set of all PERF_CTL bits (for each counter).
#[derive(Clone, Copy, Debug)]
pub struct PerfCtlDescriptor(pub [Option<usize>; 6]);
impl PerfCtlDescriptor {
    pub fn new() -> Self {
        Self([None; 6])
    }
    pub fn get(&self, idx: usize) -> u64 {
        if let Some(val) = self.0[idx] { val as u64 } else { 0 }
    }
    pub fn clear_all(&mut self) {
        self.0 = [None; 6];
    }
    pub fn clear(&mut self, idx: usize) {
        assert!(idx < 6);
        self.0[idx] = None;
    }
    pub fn set(mut self, idx: usize, x: PerfCtl) -> Self {
        assert!(idx < 6);
        self.0[idx] = Some(x.0);
        self
    }
}

#[derive(Clone, Copy, Debug)]
pub enum HostGuestBits {
    All       = 0b00,
    SVMEGuest = 0b01,
    SVMEHost  = 0b10,
    SVMEAll   = 0b11,
}
#[derive(Clone, Copy, Debug)]
pub enum OSUserBits {
    None = 0b00,
    User = 0b01,
    OS   = 0b10,
    All  = 0b11,
}

/// Wrapper type for a set of PERF_CTL bits.
#[repr(transparent)]
#[derive(Clone, Copy, Debug)]
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

    pub fn clear(&mut self) { 
        self.0 = 0;
    }
    pub fn set_hostguest(&mut self, x: HostGuestBits) {
        self.0 = (self.0 & !Self::HOSTGUEST_MASK) | (x as usize) << 40;
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
    pub fn set_osuser(&mut self, x: OSUserBits) {
        self.0 = (self.0 & !Self::OSUSER_MASK) | (x as usize) << 16
    }
    pub fn set_unit_mask(&mut self, x: u8) {
        let x = x as usize;
        self.0 = (self.0 & !Self::UNITMASK_MASK) | (x & 0b11111111) << 8
    }
}

//#[cfg(test)]
//mod test {
//    use crate::pmc::*;
//    #[test]
//    fn test() {
//        let mut x = PerfCtl(0);
//        x.set_hostguest(0b00);
//        x.set_event_select(0x76);
//        x.set_en(true);
//        x.set_int(true);
//        x.set_osuser(0b11);
//        println!("{:016x}", x.0);
//    }
//}
//

