use crate::analyze::*;
use crate::label::*;
use ansi_term::Style;
use anyhow::*;
use std::fmt;
use std::io::Write;

pub fn render(
    measurements: &Measurements,
    diffs: impl Iterator<Item = (Bench, Bench, Vec<DiffCI>)>,
    significance: f64,
) -> Result<Vec<u8>> {
    let mut out = tabwriter::TabWriter::new(Vec::<u8>::new());

    // Print the summary table
    write!(out, "benchmark\tsamples")?;
    for metric in all_metrics() {
        write!(out, "\t{:^20}", metric.to_string())?;
    }
    writeln!(out)?;
    for bench in all_benches() {
        let count = measurements.bench_stats(bench)[0].count();
        write!(out, "{}\t{}", bench, count)?;
        for stats in measurements.bench_stats(bench) {
            write!(out, "\t{:^20.3}", stats.mean())?;
        }
        writeln!(out)?;
    }
    writeln!(out)?;

    // Print the diff tables
    write!(out, "from..\t..to")?;
    for metric in all_metrics() {
        write!(out, "\t{:^20}", metric.to_string())?;
    }
    writeln!(out)?;
    for (from, to, diff) in diffs {
        write!(out, "{}\t{}", from, to)?;
        // write!(out, "{}% CI", significance)?;
        for ci in diff.iter() {
            write!(out, "\t{}", fmt_ci(ci.interval(significance / 100.)))?;
        }
        writeln!(out)?;
    }
    writeln!(
        out,
        "\nThe row from x to y shows the {}% CIs of (ȳ - x̄) / x̄ for each metric",
        significance
    )?;

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
