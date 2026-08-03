use core::sync::atomic::Ordering;

use crate::pure::Command;

use super::COMMAND;

pub fn submit_command(command: Command) {
    COMMAND.store(command.bits(), Ordering::Relaxed);
}
