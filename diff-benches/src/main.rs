use rolling_stats::Stats;
use std::process::Command;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Options {
    #[structopt(short, long)]
    csv_out: bool,
    benches: Vec<String>,
}

fn main() {
    let opts = Options::from_args();
    diff(&opts.benches);
}

fn diff(benches: &[String]) -> ConfidenceInterval {
    let mut all_measurements = Vec::<(usize, f64)>::new();
    loop {
        let idx = rand::random::<usize>() % benches.len();
        all_measurements.push((idx, run_bench(&benches[idx])));
        let ci = confidence_interval(&all_measurements);
        if ci.end - ci.start < THRESHOLD {
            return ci;
        }
    }
}

fn run_bench(cmd: &str) -> f64 {
    let out = Command::new("/bin/sh")
        .arg("-c")
        .arg(cmd)
        .output()
        .unwrap()
        .stdout;
    serde_json::from_slice(&out).unwrap()
}

type ConfidenceInterval = std::ops::Range<f64>;

const THRESHOLD: f64 = 1.0;
const SIG_LEVEL: f64 = 1.0;

fn summarize(xs: &[(usize, f64)]) -> Vec<Stats<f64>> {
    let mut stats = vec![];
    for (i, x) in xs {
        if *i >= stats.len() {
            stats.resize_with(i + 1, Stats::new)
        }
        stats[*i].update(*x);
    }
    stats
}
