use crate::diff;
use crate::label::*;
use crate::pretty;
use crate::summarize;
use anyhow::*;
use log::*;
use std::collections::{BTreeMap, BTreeSet};
use std::fs::File;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
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
    /// The target CI width.  Applies to the 95% CI; units are percent of base.
    #[structopt(long)]
    threshold: Option<f64>,
    #[structopt(long)]
    timeout: Option<f64>,
    #[structopt(long)]
    deny_positive: bool,
}

// The cbdr pipeline goes:
//
// sample -> summarize -> rate-limit -> diff -> pretty -> check-finished
//
// This subcommand just runs it all in one process
pub fn all_the_things(opts: Options) -> Result<()> {
    let mut last_print = Instant::now();
    let start_time = Instant::now();
    let mut diff = diff::State::new(opts.diff.pairs());
    let outfile: Option<File> = opts.out.map(File::create).transpose()?;
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
        } else if last_print.elapsed() > Duration::from_millis(100) {
            last_print = Instant::now();
            summarize.update(label, values.into_iter());
            diff.update(&summarize.all_measurements);
            pretty.print(&diff.diffs)?;

            // Check to see if we're finished
            if let Some(threshold) = opts.threshold {
                let worst = diff
                    .diffs
                    .iter()
                    .flat_map(|diff| stats.iter().map(move |stat| *diff.cis.get(stat)?))
                    .map(|x| x.map_or(std::f64::INFINITY, |x| x.r95_pc()))
                    .fold(std::f64::NEG_INFINITY, f64::max);
                if worst < threshold {
                    break;
                } else {
                    info!("Threshold not reached: {}% > {}%", worst, threshold);
                }
            }
            if opts
                .timeout
                .map_or(false, |t| start_time.elapsed() > Duration::from_secs_f64(t))
            {
                break;
            }
        }
    }

    if opts.deny_positive
        && diff
            .diffs
            .iter()
            .flat_map(|diff| diff.cis.values())
            .flatten()
            .any(|ci| ci.delta() > ci.r95)
    {
        bail!("Stat increased!");
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
        run_bench_with(bench, label)
    } else {
        // run_bench_in_shell(label)
        time_bench_in_shell(label)
    }
}

fn run_bench_with(bench: &str, label: &Label) -> Result<BTreeMap<String, f64>> {
    let out = Command::new(bench)
        .arg(&label)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?
        .wait_with_output()?;
    serde_json::from_slice(&out.stdout)
        .with_context(|| String::from_utf8_lossy(&out.stderr).into_owned())
}

fn run_bench_in_shell(label: &Label) -> Result<BTreeMap<String, f64>> {
    let out = Command::new("/bin/sh")
        .arg("-c")
        .arg(&label)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()?
        .wait_with_output()?;
    serde_json::from_slice(&out.stdout)
        .with_context(|| String::from_utf8_lossy(&out.stderr).into_owned())
}

fn time_bench_in_shell(label: &Label) -> Result<BTreeMap<String, f64>> {
    let mut cmd = Command::new("/bin/sh");
    cmd.arg("-c")
        .arg(&label)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    let ret = time_in_shell_posix(cmd)?;
    #[cfg(not(unix))]
    let ret = time_in_shell_fallback(cmd)?;
    Ok(ret)
}

#[allow(unused)]
fn time_in_shell_fallback(mut cmd: Command) -> Result<BTreeMap<String, f64>> {
    let ts = Instant::now();
    cmd.spawn()?.wait()?;
    let d = ts.elapsed();

    let mut ret = BTreeMap::default();
    ret.insert("wall_time".into(), d.as_secs_f64());
    Ok(ret)
}

#[cfg(unix)]
fn time_in_shell_posix(mut cmd: Command) -> Result<BTreeMap<String, f64>> {
    // times(2) and sysconf(2) are both POSIX
    let mut tms_before = libc::tms {
        tms_utime: 0,
        tms_stime: 0,
        tms_cutime: 0,
        tms_cstime: 0,
    };
    let mut tms_after = tms_before.clone();

    unsafe { libc::times(&mut tms_before as *mut libc::tms) };
    let ts = Instant::now();
    cmd.spawn()?.wait()?;
    let d = ts.elapsed();
    unsafe { libc::times(&mut tms_after as *mut libc::tms) };

    let ticks_per_sec = unsafe { libc::sysconf(libc::_SC_CLK_TCK) } as f64;
    let utime = (tms_after.tms_cutime - tms_before.tms_cutime) as f64 / ticks_per_sec;
    let stime = (tms_after.tms_cstime - tms_before.tms_cstime) as f64 / ticks_per_sec;

    let mut ret = BTreeMap::default();
    ret.insert("wall_time".into(), d.as_secs_f64());
    ret.insert("user_time".into(), utime);
    ret.insert("sys_time".into(), stime);
    Ok(ret)
}
