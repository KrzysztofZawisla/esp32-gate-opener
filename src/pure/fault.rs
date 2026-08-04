use bitflags::bitflags;

bitflags! {
    #[derive(Clone, Copy, Debug, PartialEq, Eq)]
    pub struct Fault: u8 {
        const BATTERY = 1 << 0;
        const SENSOR = 1 << 1;
    }
}
