use core::sync::atomic::Ordering;

use super::OBSTACLE;

pub fn obstacle() -> bool {
    OBSTACLE.load(Ordering::Relaxed)
}
