use core::sync::atomic::Ordering;

use crate::pure::Command;

use super::COMMAND;

pub fn take_command() -> Command {
    Command::from_raw(COMMAND.swap(Command::None.bits(), Ordering::Relaxed))
}
