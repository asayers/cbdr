use crate::label::*;
use log::*;

pub struct Measurements {
    msmts: Vec<RollingStats>,
    stride: usize, // a global constant, cached here for speed
}

impl Default for Measurements {
    fn default() -> Measurements {
        Measurements {
            msmts: vec![],
            stride: all_metrics().count(),
        }
    }
}

impl Measurements {
    pub fn bench_stats(&self, bench: Bench) -> &[RollingStats] {
        &self.msmts[bench.0 * self.stride..(bench.0 + 1) * self.stride]
    }

    fn bench_stats_mut(&mut self, bench: Bench) -> &mut [RollingStats] {
        let start = self.stride * bench.0;
        let end = self.stride * (bench.0 + 1);
        if self.msmts.len() < end {
            self.msmts.resize_with(end, RollingStats::default);
        }
        &mut self.msmts[start..end]
    }

    pub fn update(&mut self, bench: Bench, new_measurements: impl Iterator<Item = f64>) {
        for (stats, msmt) in self.bench_stats_mut(bench).iter_mut().zip(new_measurements) {
            stats.update(msmt);
        }
    }

    pub fn diff(&self, from: Bench, to: Bench) -> Vec<DiffCI> {
        self.bench_stats(from)
            .iter()
            .copied()
            .zip(self.bench_stats(to).iter().copied())
            .map(|(from, to)| DiffCI(from.into(), to.into()))
            .collect::<Vec<_>>()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct RollingStats {
    /// the number of samples seen so far
    count: usize,
    /// the mean of the entire dataset
    mean: f64,
    /// the squared distance from the mean
    m2: f64,
}

impl RollingStats {
    pub fn update(&mut self, x: f64) {
        // Welford's online algorithm
        self.count += 1;
        let delta1 = x - self.mean; // diff from the old mean
        self.mean += delta1 / self.count as f64;
        let delta2 = x - self.mean; // diff from the new mean
        self.m2 += delta1 * delta2;
    }

    pub fn count(self) -> usize {
        self.count
    }

    pub fn mean(self) -> f64 {
        if self.count == 0 {
            std::f64::NAN
        } else {
            self.mean
        }
    }

    pub fn sample_var(self) -> f64 {
        if self.count <= 1 {
            std::f64::NAN
        } else {
            self.m2 / (self.count - 1) as f64
        }
    }
}

impl Into<confidence::Stats> for RollingStats {
    fn into(self) -> confidence::Stats {
        confidence::Stats {
            count: self.count,
            mean: self.mean(),
            var: self.sample_var(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct DiffCI(pub confidence::Stats, pub confidence::Stats);
impl DiffCI {
    /// The confiendence interval for the difference of the means, given as
    /// a percentage of the first mean.
    pub fn interval(self, sig_level: f64) -> (f64, f64) {
        let delta = self.1.mean - self.0.mean;
        let width = self.width(sig_level);
        let left = 100. * (delta - width) / self.0.mean;
        let right = 100. * (delta + width) / self.0.mean;
        (left, right)
    }

    fn width(self, sig_level: f64) -> f64 {
        confidence::confidence_interval(sig_level, self.0, self.1).unwrap_or_else(|e| {
            match e {
                confidence::Error::NotEnoughData => (), // we expect some of these; ignore
                e => warn!("Skipping bad stats: {} ({:?} {:?})", e, self.0, self.1),
            };
            std::f64::NAN
        })
    }
}
