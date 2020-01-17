use crate::label::*;
use log::*;

pub struct Measurements {
    msmts: Vec<confidence::StatsBuilder>,
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
    pub fn bench_stats(&self, bench: Bench) -> &[confidence::StatsBuilder] {
        &self.msmts[bench.0 * self.stride..(bench.0 + 1) * self.stride]
    }

    fn bench_stats_mut(&mut self, bench: Bench) -> &mut [confidence::StatsBuilder] {
        let start = self.stride * bench.0;
        let end = self.stride * (bench.0 + 1);
        if self.msmts.len() < end {
            self.msmts
                .resize_with(end, confidence::StatsBuilder::default);
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
