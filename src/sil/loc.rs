#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocationType {
    VReg(u8),
    Local(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum LocationContext {
    Read,
    Write
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Location {
    pub l_type: LocationType,
    pub ctx: LocationContext,
}

impl Location {
    pub fn new(l_type: LocationType, ctx: LocationContext) -> Location {
        Self { l_type, ctx }
    }

    pub fn vreg(n: u8, ctx: LocationContext) -> Location {
        Self { l_type: LocationType::VReg(n), ctx }
    }

    pub fn local(n: u32, ctx: LocationContext) -> Location {
        Self { l_type: LocationType::Local(n), ctx }
    }
}