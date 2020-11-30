use crate::label::*;
use crate::{pretty, term_paint};
use anyhow::*;
use log::*;
use std::time::*;
use structopt::*;

/// For each pair of benchmarks (x and y), shows, for each metric (̄x
/// and ̄y), the CI of (̄y - ̄x) / ̄x
#[derive(StructOpt)]
pub struct Options {
    /// The significance level of the confidence intervals
    #[structopt(long, short, default_value = "99.9")]
    significance: f64,
    // /// The target CI width.  Applies to the 95% CI; units are percent of base.
    // #[structopt(long)]
    // threshold: Option<f64>,
    #[structopt(long)]
    deny_positive: bool,
    /// A "base" label.  If specified, all labels will be compared to this.
    #[structopt(long)]
    pub base: Option<String>,
    /// Benchs to compare.  If "base" is not specified, they'll be compared
    /// consecutively.
    pub labels: Vec<String>,
}
impl Options {
    pub fn labels_in_order<'a>(&'a self) -> Box<dyn Iterator<Item = Bench> + 'a> {
        if self.labels.is_empty() {
            Box::new(all_benches())
        } else {
            Box::new(self.labels.iter().map(|x| Bench::from(x.as_str())))
        }
    }
    pub fn pairs<'a>(&'a self) -> Box<dyn Iterator<Item = (Bench, Bench)> + 'a> {
        if let Some(base) = &self.base {
            let base = Bench::from(base.as_str());
            Box::new(
                self.labels_in_order()
                    .filter(move |x| *x != base)
                    .map(move |x| (base, x)),
            )
        } else {
            Box::new(self.labels_in_order().zip(self.labels_in_order().skip(1)))
        }
    }
}

// summarize -> rate-limit -> diff -> pretty print
pub fn analyze(opts: Options) -> Result<()> {
    if opts.significance < 0. || opts.significance > 100. {
        bail!("Significance level must be between 0 and 100");
    }
    if opts.significance < 1. {
        warn!("Significance level is given as a percentage");
    }
    let mut rdr = csv::Reader::from_reader(std::io::stdin());
    let mut headers = rdr.headers().unwrap().into_iter();
    let first = headers.next().unwrap();
    info!("Assuming \"{}\" column is the benchmark name", first);
    init_metrics(headers.map(|x| x.to_string()).collect());
    let mut measurements = Measurements::default();

    let mut stdout = term_paint::Painter::new()?;

    let mut last_print = Instant::now();
    for row in rdr.into_records() {
        let row = row?;
        let mut row = row.into_iter();
        let bench = Bench::from(row.next().unwrap());
        let values = row.map(|x| x.parse().unwrap());
        measurements.update(bench, values);

        if last_print.elapsed() > Duration::from_millis(100) {
            last_print = Instant::now();
            let diffs = opts.pairs().map(|(from, to)| {
                let diff = measurements.diff(from, to);
                (from, to, diff)
            });
            let out = pretty::render(&measurements, diffs, opts.significance)?;
            stdout.print(&out)?;

            // // Check to see if we're finished
            // if let Some(threshold) = opts.threshold {
            //     let worst = diff
            //         .diffs
            //         .iter()
            //         .flat_map(|diff| stats.iter().map(move |stat| *diff.cis.get(stat)?))
            //         .map(|x| x.map_or(std::f64::INFINITY, |x| x.r95_pc()))
            //         .fold(std::f64::NEG_INFINITY, f64::max);
            //     if worst < threshold {
            //         break;
            //     } else {
            //         info!("Threshold not reached: {}% > {}%", worst, threshold);
            //     }
            // }
        }
    }

    // Print the last set of diffs
    let diffs = opts.pairs().map(|(from, to)| {
        let diff = measurements.diff(from, to);
        (from, to, diff)
    });
    let out = pretty::render(&measurements, diffs, opts.significance)?;
    stdout.print(&out)?;

    if opts.deny_positive {
        for (from, to) in opts.pairs() {
            for (idx, ci) in measurements.diff(from, to).into_iter().enumerate() {
                let metric = Metric(idx);
                if ci.interval(0.95).0 > 0. {
                    bail!("{}..{}: {} increased!", from, to, metric);
                }
            }
        }
    }

    Ok(())
}

pub struct Measurements {
    msmts: Vec<behrens_fisher::StatsBuilder>,
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
    pub fn bench_stats(&self, bench: Bench) -> &[behrens_fisher::StatsBuilder] {
        &self.msmts[bench.0 * self.stride..(bench.0 + 1) * self.stride]
    }

    pub fn update(&mut self, bench: Bench, new_measurements: impl Iterator<Item = f64>) {
        let start = self.stride * bench.0;
        let end = self.stride * (bench.0 + 1);
        if self.msmts.len() < end {
            self.msmts
                .resize_with(end, behrens_fisher::StatsBuilder::default);
        }
        for (stats, msmt) in self.msmts[start..end].iter_mut().zip(new_measurements) {
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
pub struct DiffCI(pub behrens_fisher::Stats, pub behrens_fisher::Stats);
impl DiffCI {
    /// The confidence interval for the difference of the means, given as
    /// a percentage of the first mean.
    pub fn interval(self, sig_level: f64) -> (f64, f64) {
        let width =
            behrens_fisher::confidence_interval(sig_level, self.0, self.1).unwrap_or(std::f64::NAN);
        let delta = self.1.mean - self.0.mean;
        let left = 100. * (delta - width) / self.0.mean;
        let right = 100. * (delta + width) / self.0.mean;
        (left, right)
    }
}
