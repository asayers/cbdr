use crate::diff::*;
use crate::label::*;
use confidence::*;

pub struct Measurements {
    msmts: Vec<Statistics>,
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
    pub fn get(&self, bench: Bench, metric: Metric) -> Option<&Statistics> {
        assert!(metric.0 < self.stride);
        let idx = bench.0 * self.stride + metric.0;
        self.msmts.get(idx)
    }
    pub fn update(&mut self, bench: Bench, new_measurements: impl Iterator<Item = f64>) {
        for (stats, msmt) in self
            .bench_stats_mut(bench)
            .into_iter()
            .zip(new_measurements)
        {
            stats.0 += 1;
            stats.1.update(msmt);
        }
    }
    pub fn bench_stats(&self, bench: Bench) -> &[Statistics] {
        &self.msmts[bench.0 * self.stride..(bench.0 + 1) * self.stride]
    }
    pub fn bench_stats_mut(&mut self, bench: Bench) -> &mut [Statistics] {
        let start = self.stride * bench.0;
        let end = self.stride * (bench.0 + 1);
        if self.msmts.len() < end {
            self.msmts.resize_with(end, Statistics::default);
        }
        &mut self.msmts[start..end]
    }
    pub fn iter_label(&self, bench: Bench) -> impl Iterator<Item = (Metric, &Statistics)> {
        self.bench_stats(bench)
            .into_iter()
            .enumerate()
            .map(|(idx, stats)| (Metric(idx), stats))
    }

    pub fn diff(&self, from: Bench, to: Bench) -> Diff {
        Diff(
            self.bench_stats(from)
                .into_iter()
                .zip(self.bench_stats(to).into_iter())
                .map(|(from, to)| DiffCI {
                    stats_x: from.into(),
                    stats_y: to.into(),
                })
                .collect::<Vec<_>>(),
        )
    }
}

#[derive(Clone, Debug)]
pub struct Statistics(pub usize, pub rolling_stats::Stats<f64>);
impl Default for Statistics {
    fn default() -> Statistics {
        Statistics(0, rolling_stats::Stats::new())
    }
}
impl Into<Stats> for &Statistics {
    fn into(self) -> Stats {
        Stats {
            count: self.0,
            mean: self.1.mean,
            std_dev: self.1.std_dev,
        }
    }
}
