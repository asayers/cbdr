use crate::diff::*;
use crate::label::*;
use confidence::*;

pub struct Measurements {
    msmts: Vec<Statistics>,
    stride: usize,
}

impl Measurements {
    pub fn new(all_metrics: &[Metric]) -> Measurements {
        Measurements {
            msmts: vec![],
            stride: all_metrics.len(),
        }
    }
    pub fn get(&self, bench: Bench, metric: Metric) -> Option<&Statistics> {
        assert!(metric.0 < self.stride);
        let idx = bench.0 * self.stride + metric.0;
        self.msmts.get(idx)
    }
    pub fn get_mut(&mut self, bench: Bench, metric: Metric) -> &mut Statistics {
        assert!(metric.0 < self.stride);
        let idx = bench.0 * self.stride + metric.0;
        if idx >= self.msmts.len() {
            self.msmts.resize_with(idx + 1, Statistics::default);
        }
        self.msmts.get_mut(idx).unwrap()
    }
    pub fn update(&mut self, label: Bench, values: impl Iterator<Item = (Metric, f64)>) {
        for (stat, value) in values {
            let Statistics(count, x) = self.get_mut(label, stat);
            *count += 1;
            x.update(value);
        }
    }
    pub fn bench_stats(&self, bench: Bench) -> &[Statistics] {
        &self.msmts[bench.0 * self.stride..(bench.0 + 1) * self.stride]
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
    pub fn labels(&self) -> impl Iterator<Item = Bench> + Clone {
        let mut scores: Vec<(usize, f64)> = self
            .msmts
            .chunks(self.stride)
            .map(|xs| xs.into_iter().map(|stats| stats.1.mean).sum::<f64>())
            .enumerate()
            .collect();
        scores.sort_by(|x, y| x.1.partial_cmp(&y.1).unwrap_or(std::cmp::Ordering::Equal));
        scores.into_iter().map(|x| Bench(x.0))
    }

    pub fn guess_pairs(&self) -> Vec<(Bench, Bench)> {
        let labels = self.labels();
        labels.clone().zip(labels.skip(1)).collect()
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
