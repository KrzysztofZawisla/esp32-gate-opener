use esp_idf_hal::gpio::{AnyIOPin, AnyOutputPin, Input, Output, PinDriver};

pub struct GatePins {
    pub open_relay: PinDriver<'static, AnyOutputPin, Output>,
    pub close_relay: PinDriver<'static, AnyOutputPin, Output>,
    pub lamp_green: PinDriver<'static, AnyOutputPin, Output>,
    pub lamp_red: PinDriver<'static, AnyOutputPin, Output>,
    pub open_sensor: PinDriver<'static, AnyIOPin, Input>,
    pub closed_sensor: PinDriver<'static, AnyIOPin, Input>,
    pub obstacle_sensor: PinDriver<'static, AnyIOPin, Input>,
    pub obstacle_active_level: bool,
}

mod close_gate;
mod grace_ms;
mod handle_command;
mod motion_timeout_ms;
mod open_gate;
mod pulse_interruptible;
mod pulse_ms;
mod reverse_to_open;
mod set_lamp;
mod wait_interruptible;

pub use handle_command::handle_command;
pub use set_lamp::set_lamp;

pub(crate) use close_gate::close_gate;
pub(crate) use grace_ms::grace_ms;
pub(crate) use motion_timeout_ms::motion_timeout_ms;
pub(crate) use open_gate::open_gate;
pub(crate) use pulse_interruptible::pulse_interruptible;
pub(crate) use pulse_ms::pulse_ms;
pub(crate) use reverse_to_open::reverse_to_open;
pub(crate) use wait_interruptible::wait_interruptible;