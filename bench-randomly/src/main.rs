use std::collections::{BTreeSet, HashMap};
use std::io::Write;
use std::process::Command;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Options {
    benches: Vec<String>,
}

fn main() {
    match main2() {
        Ok(()) => (),
        Err(ref e) if e.kind() == std::io::ErrorKind::BrokenPipe => (),
        _ => std::process::exit(1),
    }
}
fn main2() -> Result<(), Box<std::io::Error>> {
    let opts = Options::from_args();

    let mut stats = BTreeSet::new();
    for (idx, bench) in opts.benches.iter().enumerate() {
        eprintln!("Warming up {}...", idx);
        let out = Command::new("/bin/sh")
            .arg("-c")
            .arg(bench)
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
    stdout.write_all(b"idx")?;
    for stat in &stats {
        write!(stdout, ",{}", stat)?;
    }
    stdout.write_all(b"\n")?;
    loop {
        let idx = rand::random::<usize>() % opts.benches.len();
        let out = Command::new("/bin/sh")
            .arg("-c")
            .arg(&opts.benches[idx])
            .output()
            .unwrap()
            .stdout;
        let results: HashMap<String, f64> = serde_json::from_slice(&out).unwrap();
        write!(stdout, "{}", idx)?;
        for stat in &stats {
            write!(stdout, ",{}", results.get(stat).unwrap_or(&std::f64::NAN))?;
        }
        stdout.write_all(b"\n")?;
    }
}
