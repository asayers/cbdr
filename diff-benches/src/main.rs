use ansi_term::{Color, Style};
use confidence::*;
use std::io::{BufWriter, Write};
use std::process::Command;
use structopt::StructOpt;

#[derive(StructOpt)]
struct Options {
    #[structopt(short, long)]
    threshold: Option<f64>,
    #[structopt(short, long, default_value = "0.95")]
    significance_level: f64,
    benches: Vec<String>,
}

fn main() {
    let opts = Options::from_args();
    let ci = diff(opts);
    println!("{:?}", ci);
}

fn diff(opts: Options) -> Result<Vec<ConfidenceInterval>, Box<dyn std::error::Error>> {
    let mut stdout = std::io::stdout();
    // let mut stdout = BufWriter::new(stdout.lock());
    // let mut stdout = tabwriter::TabWriter::new(stdout);
    let mut all_measurements = vec![vec![]; opts.benches.len()];
    let mut num_measurements = vec![0; opts.benches.len()];
    for (idx, bench) in opts.benches.iter().enumerate() {
        run_bench(bench);
        write!(stdout, "{:03?}", num_measurements)?;
        for i in 0..opts.benches.len() - 1 {
            if i == idx || i + 1 == idx {
                write!(stdout, "\t{}", Style::new().dimmed().paint("warmup"))?;
            } else {
                write!(stdout, "\t{}", Style::new().dimmed().paint("-"))?;
            }
        }
        writeln!(stdout)?;
    }
    loop {
        let idx = rand::random::<usize>() % opts.benches.len();
        let measurement = run_bench(&opts.benches[idx]);
        all_measurements[idx].push(measurement);
        num_measurements[idx] += 1;
        let stats = summarize(&all_measurements);
        let cis = stats
            .iter()
            .zip(stats.iter().skip(1))
            .map(|(x, y)| confidence_interval(opts.significance_level, *x, *y))
            .collect::<Vec<_>>();
        write!(stdout, "{:03?}", num_measurements)?;
        for ci in &cis {
            match ci {
                None => write!(
                    stdout,
                    "\t{}",
                    Style::new().dimmed().paint("insufficient data")
                )?,
                Some(ci) if ci.center - ci.radius < 0. && 0. < ci.center + ci.radius => {
                    let mut buf_center = ryu::Buffer::new();
                    let mut buf_radius = ryu::Buffer::new();
                    let center = buf_center.format(ci.center);
                    let radius = buf_radius.format(ci.radius);
                    write!(stdout, "\t{:>18} ± {:<18}", center, radius)?;
                }
                Some(ci) => write!(
                    stdout,
                    "\t{}{:.6} ± {:.6}{}",
                    Color::Yellow.prefix(),
                    ci.center,
                    ci.radius,
                    Color::Yellow.suffix()
                )?,
            }
        }
        writeln!(stdout)?;
        if cis.iter().all(|ci| match (opts.threshold, ci) {
            (Some(t), Some(ci)) => ci.radius < t,
            _ => false,
        }) {
            return Ok(cis.into_iter().map(|ci| ci.unwrap()).collect());
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

fn summarize(xss: &[Vec<f64>]) -> Vec<Stats> {
    xss.iter()
        .map(|xs| {
            let count = xs.len();
            let stats = xs.iter().fold(rolling_stats::Stats::new(), |mut stats, x| {
                stats.update(*x);
                stats
            });
            Stats {
                count,
                mean: stats.mean,
                std_dev: stats.std_dev,
            }
        })
        .collect()
}
