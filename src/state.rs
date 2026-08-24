//! Shared runtime state: depths + active layer.

use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Mutex;

pub const NUM_KEYS: usize = 20;

static DEPTHS: Mutex<[u8; NUM_KEYS]> = Mutex::new([0u8; NUM_KEYS]);
static LAYER: AtomicUsize = AtomicUsize::new(0);

pub fn set_depths(d: &[u8; NUM_KEYS]) {
    if let Ok(mut g) = DEPTHS.lock() {
        *g = *d;
    }
}

pub fn get_depths() -> [u8; NUM_KEYS] {
    DEPTHS.lock().map(|g| *g).unwrap_or([0u8; NUM_KEYS])
}

pub fn layer() -> usize {
    LAYER.load(Ordering::Relaxed).min(1)
}

pub fn set_layer(l: usize) {
    LAYER.store(l.min(1), Ordering::Relaxed);
}

pub fn toggle_layer() -> usize {
    let next = 1 - layer();
    set_layer(next);
    next
}
