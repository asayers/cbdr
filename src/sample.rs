use anyhow::*;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io::Write;
use std::process::{Command, Stdio};
use std::time::*;
use structopt::*;

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
            benches.extend(self.targets.into_iter().map(|x| Benchmark::Prog(x)));
        }
        benches
    }
}

pub fn sample(opts: Options) -> Result<()> {
    let benches = opts.benchmarks();
    let stats = warm_up(&benches)?;
    let mut stdout = CsvWriter::new(std::io::stdout(), stats.iter())?;
    loop {
        let idx = rand::random::<usize>() % benches.len();
        let bench = &benches[idx];
        let x = run_bench(bench).map(|x| (bench.clone(), x));
        let (bench, values) = x?;
        stdout.write_csv(&bench.to_string(), &values)?;
    }
}

struct CsvWriter<T> {
    out: T,
    stats: Vec<String>,
}
impl<T: Write> CsvWriter<T> {
    fn new<'a>(mut out: T, stats: impl Iterator<Item = &'a String>) -> Result<CsvWriter<T>> {
        out.write_all(b"target")?;
        let stats = stats
            .map(|x| {
                write!(out, ",{}", x)?;
                Ok(x.to_string())
            })
            .collect::<Result<Vec<_>>>()?;
        out.write_all(b"\n")?;
        Ok(CsvWriter { out, stats })
    }
    fn write_csv(&mut self, target: &String, values: &BTreeMap<String, f64>) -> Result<()> {
        write!(self.out, "{}", target)?;
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
                .stdout(Stdio::piped())
                .stderr(Stdio::piped());
            #[cfg(unix)]
            let ret = time_in_shell_posix(cmd)?;
            #[cfg(not(unix))]
            let ret = time_in_shell_fallback(cmd)?;
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
