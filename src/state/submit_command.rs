use core::sync::atomic::Ordering;

use super::COMMAND;

pub fn submit_command(cmd: u8) {
    COMMAND.store(cmd, Ordering::Relaxed);
}