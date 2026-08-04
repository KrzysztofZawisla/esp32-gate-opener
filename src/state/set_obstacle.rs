use core::sync::atomic::Ordering;

use super::OBSTACLE;

pub fn set_obstacle(active: bool) {
    OBSTACLE.store(active, Ordering::Relaxed);
}
