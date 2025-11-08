use serde::{Deserialize, Serialize};

use crate::F;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Sampling {
    pub level: SamplingLevel,
    pub random_offsets: bool,
}

impl Sampling {
    pub fn sampling_points(&self) -> impl Iterator<Item = (F, F)> {
        const PHI: F = 1.618033988749895;
        const EPS: F = 0.5;

        let n = self.sample_count();

        (0..n).map(move |i| {
            (
                (i as F / PHI) % 1.,
                (i as F + EPS) / ((n - 1) as F + 2. * EPS),
            )
        })
    }

    pub fn sample_count(&self) -> usize {
        match self.level {
            SamplingLevel::Raw => 1,
            SamplingLevel::Exploration => 8,
            SamplingLevel::Low => 21,
            SamplingLevel::Medium => 34,
            SamplingLevel::High => 55,
            SamplingLevel::Ultra => 89,
            SamplingLevel::Extreme => 144,
            SamplingLevel::Custom(n) => n,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SamplingLevel {
    Raw,
    Exploration,
    Low,
    Medium,
    High,
    Ultra,
    Extreme,
    Custom(usize),
}

pub fn map_point_with_offset(x: F, y: F, offset_x: F, offset_y: F) -> (F, F) {
    #[inline]
    fn tent(t: F) -> F {
        let t = 2. * t - 1.;
        if t != 0. { t - t.signum() } else { 1. }.abs()
    }

    let (x, y) = ((x + offset_x) % 1., (y + offset_y) % 1.);

    const R: F = 1.5;
    let (x, y) = (R * tent(x), R * tent(y));

    (x, y)
}
