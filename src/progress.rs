use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};

#[derive(Debug, Clone)]
pub struct Progress {
    progress: Arc<AtomicUsize>,
    pub total: usize,
}

impl Progress {
    pub fn new(total: usize) -> Self {
        Progress {
            progress: Arc::new(AtomicUsize::new(0)),
            total,
        }
    }

    pub fn add(&self, n: usize) {
        // Using atomic::Ordering::Relaxed because we don't really
        // care about the order `progress` is updated. As long as it
        // is updated it should be fine :>
        self.progress.fetch_add(n, Ordering::Relaxed);
    }

    pub fn get(&self) -> usize {
        self.progress.load(Ordering::Relaxed)
    }
    // Outputs progress in range (0,1)
    pub fn get_progress(&self) -> f32 {
        self.get() as f32 / self.total as f32
    }
}
