use std::collections::{BTreeSet, HashMap};
use std::io::Write;
use std::process::Command;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Options {
    bench_prog: String,
    labels: Vec<String>,
}

pub fn sample(opts: Options) -> Result<(), Box<dyn std::error::Error>> {
    let mut stats = BTreeSet::new();
    for label in &opts.labels {
        eprintln!("Warming up {}...", label);
        let out = Command::new(&opts.bench_prog)
            .arg(label)
            .output()
            .unwrap()
            .stdout;
        let results: HashMap<String, f64> = match serde_json::from_slice(&out) {
            Ok(x) => x,
            Err(e) => {
                eprintln!("{}", String::from_utf8_lossy(&out));
                panic!(e);
            }
        };
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
        let out = Command::new(&opts.bench_prog)
            .arg(label)
            .output()
            .unwrap()
            .stdout;
        let results: HashMap<String, f64> = serde_json::from_slice(&out).unwrap();
        write!(stdout, "{}", label)?;
        for stat in &stats {
            write!(stdout, ",{}", results.get(stat).unwrap_or(&std::f64::NAN))?;
        }
        stdout.write_all(b"\n")?;
    }
}
