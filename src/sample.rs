use anyhow::*;
use std::collections::{BTreeSet, HashMap};
use std::io::Write;
use std::process::Command;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Options {
    bench_prog: String,
    labels: Vec<String>,
}

fn run_bench(opts: &Options, label: &str) -> Result<HashMap<String, f64>> {
    let out = Command::new(&opts.bench_prog)
        .arg(label)
        .output()
        .unwrap();
    std::io::stderr().write_all(&out.stderr)?;
    serde_json::from_slice(&out.stdout)
        .with_context(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

pub fn sample(opts: Options) -> Result<()> {
    let mut stats = BTreeSet::new();
    for label in &opts.labels {
        eprintln!("Warming up {}...", label);
        let results = run_bench(&opts, label)?;
        stats.extend(results.keys().cloned());
    }

    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    stdout.write_all(b"label")?;
    for stat in &stats {
        write!(stdout, ",{}", stat)?;
    }
    stdout.write_all(b"\n")?;
    loop {
        let idx = rand::random::<usize>() % opts.labels.len();
        let label = &opts.labels[idx];
        let results = run_bench(&opts, label)?;
        write!(stdout, "{}", label)?;
        for stat in &stats {
            write!(stdout, ",{}", results.get(stat).unwrap_or(&std::f64::NAN))?;
        }
        stdout.write_all(b"\n")?;
    }
}
