#[repr(u8)]
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Status {
    Stopped = 0,
    Open = 1,
    Closed = 2,
    Opening = 3,
    Closing = 4,
    Error = 5,
}

impl Status {
    pub const fn from_raw(raw: u8) -> Self {
        match raw {
            1 => Self::Open,
            2 => Self::Closed,
            3 => Self::Opening,
            4 => Self::Closing,
            5 => Self::Error,
            _ => Self::Stopped,
        }
    }

    pub const fn bits(self) -> u8 {
        self as u8
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Stopped => "stopped",
            Self::Open => "open",
            Self::Closed => "closed",
            Self::Opening => "opening",
            Self::Closing => "closing",
            Self::Error => "error",
        }
    }
}

pub fn sensor_status(open_active: bool, closed_active: bool) -> Status {
    match (open_active, closed_active) {
        (true, true) => Status::Error,
        (true, false) => Status::Open,
        (false, true) => Status::Closed,
        (false, false) => Status::Stopped,
    }
}
