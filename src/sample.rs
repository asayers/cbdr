use anyhow::*;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io::Write;
use std::process::{Command, Stdio};
use structopt::*;
use time_cmd::*;

#[derive(StructOpt)]
pub struct Options {
    /// A benchmark script to use.  Labels will be passed as $1
    #[structopt(long, short)]
    pub bench: Option<String>,
    /// These benchmarks will be run in a shell and their output will be used to compute stats
    #[structopt(long, short)]
    pub scripts: Vec<String>,
    // #[structopt(long, short)]
    // pub script: bool,
    /// Labels to compare.  If "base" is not specified, they'll be compared
    /// consecutively.
    pub targets: Vec<String>,
}
impl Options {
    fn benchmarks(self) -> Vec<Benchmark> {
        let mut benches = self
            .scripts
            .into_iter()
            .map(|x| Benchmark::Script(x, vec![]))
            .collect::<Vec<_>>();
        if let Some(bench) = self.bench {
            benches.extend(
                self.targets
                    .into_iter()
                    .map(|x| Benchmark::Script(bench.clone(), vec![x])),
            );
        } else {
            benches.extend(self.targets.into_iter().map(Benchmark::Prog));
        }
        benches
    }
}

pub fn sample(opts: Options) -> Result<()> {
    let benches = opts.benchmarks();
    let stats = warm_up(&benches)?;
    let mut stdout = CsvWriter::new(std::io::stdout(), stats.iter())?;
    // Run the benches in-order once, so `cbdr analyze` knows the correct order
    for bench in &benches {
        let values = run_bench(bench)?;
        stdout.write_csv(&bench.to_string(), &values)?;
    }
    loop {
        let idx = rand::random::<usize>() % benches.len();
        let bench = &benches[idx];
        let values = run_bench(bench)?;
        stdout.write_csv(&bench.to_string(), &values)?;
    }
}

struct CsvWriter<T> {
    out: T,
    stats: Vec<String>,
}
impl<T: Write> CsvWriter<T> {
    fn new<'a>(mut out: T, stats: impl Iterator<Item = &'a String>) -> Result<CsvWriter<T>> {
        out.write_all(b"benchmark")?;
        let stats = stats
            .map(|x| {
                write!(out, ",{}", x)?;
                Ok(x.to_string())
            })
            .collect::<Result<Vec<_>>>()?;
        out.write_all(b"\n")?;
        Ok(CsvWriter { out, stats })
    }
    fn write_csv(&mut self, bench: &str, values: &BTreeMap<String, f64>) -> Result<()> {
        write!(self.out, "{}", bench)?;
        for stat in &self.stats {
            write!(self.out, ",{}", values.get(stat).unwrap_or(&std::f64::NAN))?;
        }
        self.out.write_all(b"\n")?;
        Ok(())
    }
}

fn warm_up(benches: &[Benchmark]) -> Result<BTreeSet<String>> {
    let mut stats = BTreeSet::new();
    for bench in benches {
        eprintln!("Warming up {}...", bench);
        let results = run_bench(bench)?;
        stats.extend(results.keys().cloned());
    }
    eprintln!();
    Ok(stats)
}

enum Benchmark {
    Prog(String),
    Script(String, Vec<String>),
}
impl fmt::Display for Benchmark {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Benchmark::Prog(x) => f.write_str(&x),
            Benchmark::Script(x, args) => write!(f, "<{} {:?}>", x, args),
        }
    }
}

fn run_bench(bench: &Benchmark) -> Result<BTreeMap<String, f64>> {
    match bench {
        Benchmark::Prog(x) => {
            let mut cmd = Command::new("/bin/sh");
            cmd.arg("-c")
                .arg(x)
                .stdout(Stdio::null())
                .stderr(Stdio::null());
            let mut ret = BTreeMap::default();
            let timings = time_cmd(cmd)?;
            ret.insert("wall time".into(), timings.wall_time.as_secs_f64());
            ret.insert("user time".into(), timings.user_time);
            ret.insert("sys time".into(), timings.sys_time);
            Ok(ret)
        }
        Benchmark::Script(script, args) => {
            let out = Command::new(script)
                .args(args)
                .stdout(Stdio::piped())
                .stderr(Stdio::piped())
                .spawn()?
                .wait_with_output()?;
            serde_json::from_slice(&out.stdout)
                .with_context(|| String::from_utf8_lossy(&out.stderr).into_owned())
        }
    }
}
