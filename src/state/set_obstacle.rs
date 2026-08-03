use core::sync::atomic::Ordering;

use super::OBSTACLE;

pub fn set_obstacle(on: bool) {
    OBSTACLE.store(on, Ordering::Relaxed);
}