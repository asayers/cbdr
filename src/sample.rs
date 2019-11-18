use crate::diff;
use anyhow::*;
use std::collections::{BTreeMap, BTreeSet};
use std::io::Write;
use std::process::Command;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Options {
    #[structopt(long, short)]
    bench: String,
    #[structopt(flatten)]
    pub labels: diff::Options,
}

fn run_bench(bench: &str, label: &str) -> Result<BTreeMap<String, f64>> {
    let out = Command::new(bench).arg(label).output().unwrap();
    std::io::stderr().write_all(&out.stderr)?; // TODO: swallow
    serde_json::from_slice(&out.stdout)
        .with_context(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

pub struct Samples {
    bench: String,
    all_labels: Vec<String>,
}
impl Samples {
    pub fn new(opts: Options) -> Result<(Samples, BTreeSet<String>)> {
        let all_labels = opts.labels.all_labels();
        let mut stats = BTreeSet::new();
        for label in &all_labels {
            eprintln!("Warming up {}...", label);
            let results = run_bench(&opts.bench, label)?;
            stats.extend(results.keys().cloned());
        }
        Ok((
            Samples {
                bench: opts.bench,
                all_labels,
            },
            stats,
        ))
    }
}
impl Iterator for Samples {
    type Item = Result<(String, BTreeMap<String, f64>)>;
    fn next(&mut self) -> Option<Self::Item> {
        let idx = rand::random::<usize>() % self.all_labels.len();
        let label = &self.all_labels[idx];
        let x = run_bench(&self.bench, label).map(|x| (label.clone(), x));
        Some(x)
    }
}

pub fn sample(opts: Options) -> Result<()> {
    let (iter, stats) = Samples::new(opts)?;
    let stdout = std::io::stdout();
    let mut stdout = stdout.lock();
    stdout.write_all(b"label")?;
    for stat in &stats {
        write!(stdout, ",{}", stat)?;
    }
    stdout.write_all(b"\n")?;
    for x in iter {
        let (label, results) = x?;
        write!(stdout, "{}", label)?;
        for stat in &stats {
            write!(stdout, ",{}", results.get(stat).unwrap_or(&std::f64::NAN))?;
        }
        stdout.write_all(b"\n")?;
    }
    Ok(())
}
