use std::sync::atomic::{AtomicUsize, Ordering};

pub struct Progress {
    progress: AtomicUsize,
    pub total: usize,
}

impl Progress {
    pub fn new(total: usize) -> Self {
        Progress {
            progress: AtomicUsize::new(0),
            total,
        }
    }

    pub fn incr(&self) {
        // Using atomic::Ordering::Relaxed because we don't really
        // care about the order `progress` is updated. As long as it
        // is updated it should be fine :>
        self.progress.fetch_add(1, Ordering::Relaxed);
    }

    pub fn get(&self) -> usize {
        self.progress.load(Ordering::Relaxed)
    }
    pub fn get_percent(&self) -> f32 {
        100. * self.get() as f32 / self.total as f32
    }
}
