use super::{persist_u16, GRACE_MS, KEY_GRACE_MS};

pub fn set_grace_ms(value: u16) -> bool {
    persist_u16(KEY_GRACE_MS, value, &GRACE_MS)
}