use crate::analyze::*;
use crate::label::*;
use ansi_term::Style;
use anyhow::*;
use std::fmt;
use std::io::Write;

pub fn render(
    measurements: &Measurements,
    diffs: impl Iterator<Item = (Bench, Bench, Vec<DiffCI>)>,
) -> Result<Vec<u8>> {
    let mut out = tabwriter::TabWriter::new(Vec::<u8>::new());

    // Print the summary table
    write!(out, "benchmark\tsamples")?;
    for metric in all_metrics() {
        write!(out, "\t{}", metric)?;
    }
    writeln!(out)?;
    for bench in all_benches() {
        let count = measurements.bench_stats(bench)[0].count();
        write!(out, "{}\t{}", bench, count)?;
        for stats in measurements.bench_stats(bench) {
            write!(out, "\t{:.3}", stats.mean())?;
        }
        writeln!(out)?;
    }

    // Print the diff tables
    for (from, to, diff) in diffs {
        writeln!(out, "\n{} vs {}:\n", from, to)?;
        for metric in all_metrics() {
            write!(out, "\t{:^20}", metric.to_string())?;
        }
        writeln!(out)?;
        for p in &[0.9999, 0.999, 0.99, 0.95, 0.5, 0.] {
            write!(out, "{}% CI", p * 100.)?;
            for ci in diff.iter() {
                write!(out, "\t{}", fmt_ci(ci.interval(*p)))?;
            }
            writeln!(out)?;
        }
    }

    let out = out.into_inner()?;
    Ok(out)
}

fn fmt_ci((l, r): (f64, f64)) -> impl fmt::Display {
    let s = format!("[{:>+6.1}% .. {:>+6.1}%]", l, r);
    if l > 0. || r < 0. {
        Style::new().bold().paint(s)
    } else {
        Style::new().dimmed().paint(s)
    }
}
