#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Command {
    None = 0,
    Open = 1,
    Close = 2,
}

impl Command {
    pub const fn from_raw(raw: u8) -> Self {
        match raw {
            1 => Self::Open,
            2 => Self::Close,
            _ => Self::None,
        }
    }

    pub const fn bits(self) -> u8 {
        self as u8
    }
}
