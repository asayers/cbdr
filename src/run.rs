use crate::diff;
use crate::label::*;
use crate::limit;
use crate::pretty;
use crate::summarize;
use anyhow::*;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::Command;
use std::time::*;
use structopt::*;

#[derive(StructOpt)]
pub struct Options {
    /// A benchmark script to use.  Labels will be passed as $1
    #[structopt(long, short)]
    pub bench: Option<String>,
    /// Write bench results to stdout (disables CI view)
    #[structopt(long)]
    stdout: bool,
    /// Write bench results to a file
    #[structopt(long, short)]
    pub out: Option<PathBuf>,
    #[structopt(flatten)]
    pub diff: diff::Options,
    #[structopt(flatten)]
    limit: limit::Options,
}

// The cbdr pipeline goes:
//
// sample -> summarize -> rate-limit -> diff -> pretty -> check-finished
//
// This subcommand just runs it all in one process
pub fn all_the_things(opts: Options) -> Result<()> {
    let mut diff = diff::State::new(opts.diff.pairs());
    let outfile: Option<File> = opts.out.map(|path| File::create(path)).transpose()?;
    let (samples, stats) = Samples::new(opts.bench, opts.diff.all_labels())?;
    let mut summarize = summarize::State::new();
    let mut pretty = pretty::State::new()?;
    let mut outfile = outfile
        .map(|outfile| CsvWriter::new(outfile, stats.iter()))
        .transpose()?;
    let mut stdout = if opts.stdout {
        Some(CsvWriter::new(std::io::stdout(), stats.iter())?)
    } else {
        None
    };
    for x in samples {
        let (label, values) = x?;
        if let Some(ref mut file) = outfile {
            file.write_csv(&label, &values)?;
        }
        if let Some(ref mut stdout) = stdout {
            stdout.write_csv(&label, &values)?;
        } else {
            summarize.update(label, values.into_iter());
            diff.update(&summarize.all_measurements);
            pretty.print(&diff.diffs)?;
            if limit::is_finished(&opts.limit, &diff.diffs, &stats) {
                break;
            }
        }
    }
    Ok(())
}

struct CsvWriter<T> {
    out: T,
    stats: Vec<String>,
}
impl<T: Write> CsvWriter<T> {
    fn new<'a>(mut out: T, stats: impl Iterator<Item = &'a String>) -> Result<CsvWriter<T>> {
        out.write_all(b"label")?;
        let stats = stats
            .map(|x| {
                write!(out, ",{}", x)?;
                Ok(x.to_string())
            })
            .collect::<Result<Vec<_>>>()?;
        out.write_all(b"\n")?;
        Ok(CsvWriter { out, stats })
    }
    fn write_csv(&mut self, label: &Label, values: &BTreeMap<String, f64>) -> Result<()> {
        write!(self.out, "{}", label)?;
        for stat in &self.stats {
            write!(self.out, ",{}", values.get(stat).unwrap_or(&std::f64::NAN))?;
        }
        self.out.write_all(b"\n")?;
        Ok(())
    }
}

pub struct Samples {
    bench: Option<String>,
    all_labels: Vec<Label>,
}
impl Samples {
    pub fn new(bench: Option<String>, labels: Vec<Label>) -> Result<(Samples, BTreeSet<String>)> {
        let mut stats = BTreeSet::new();
        for label in &labels {
            eprintln!("Warming up {}...", label);
            let results = run_bench(&bench, &label)?;
            stats.extend(results.keys().cloned());
        }
        Ok((
            Samples {
                bench,
                all_labels: labels,
            },
            stats,
        ))
    }
}
impl Iterator for Samples {
    type Item = Result<(Label, BTreeMap<String, f64>)>;
    fn next(&mut self) -> Option<Self::Item> {
        let idx = rand::random::<usize>() % self.all_labels.len();
        let label = &self.all_labels[idx];
        let x = run_bench(&self.bench, label).map(|x| (label.clone(), x));
        Some(x)
    }
}

fn run_bench(bench: &Option<String>, label: &Label) -> Result<BTreeMap<String, f64>> {
    if let Some(bench) = bench {
        run_manual_bench(bench, label)
    } else {
        run_default_bench(label)
    }
}

fn run_manual_bench(bench: &str, label: &Label) -> Result<BTreeMap<String, f64>> {
    let out = Command::new(bench).arg(&label).output()?;
    std::io::stderr().write_all(&out.stderr)?; // TODO: swallow
    serde_json::from_slice(&out.stdout)
        .with_context(|| String::from_utf8_lossy(&out.stdout).into_owned())
}

fn run_default_bench(label: &Label) -> Result<BTreeMap<String, f64>> {
    use std::process::Stdio;
    let ts = Instant::now();
    Command::new("/bin/sh")
        .arg("-c")
        .arg(&label)
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()?
        .wait()?;
    let wall_time = ts.elapsed();
    Ok(vec![("wall_clock".into(), wall_time.as_secs_f64())]
        .into_iter()
        .collect())
}

#[cfg(test)]
mod tests {
    #[test]
    fn test() {
        assert_eq!(1 + 1, 2)
    }
}
