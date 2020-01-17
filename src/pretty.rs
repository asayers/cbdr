use crate::label::*;
use crate::summarize::*;
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
        writeln!(
            out,
            "\t{:^20}\t{:^20}\t{:^20}\t{:^20}",
            "95% CI", "99% CI", "99.9% CI", "99.99% CI",
        )?;
        for (idx, ci) in diff.iter().enumerate() {
            let metric = Metric(idx);
            writeln!(
                out,
                "    {}\t{}\t{}\t{}\t{}",
                metric,
                fmt_ci(ci.interval(0.95)),
                fmt_ci(ci.interval(0.99)),
                fmt_ci(ci.interval(0.999)),
                fmt_ci(ci.interval(0.9999)),
            )?;
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
