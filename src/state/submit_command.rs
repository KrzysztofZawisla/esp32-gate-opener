use core::sync::atomic::Ordering;

use super::COMMAND;

pub fn submit_command(command: u8) {
    COMMAND.store(command, Ordering::Relaxed);
}
