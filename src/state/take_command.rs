use core::sync::atomic::Ordering;

use crate::config::CMD_NONE;

use super::COMMAND;

pub fn take_command() -> u8 {
    COMMAND.swap(CMD_NONE, Ordering::Relaxed)
}