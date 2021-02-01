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
) -> Result<String> {
    let mut out = tabwriter::TabWriter::new(Vec::<u8>::new());

    let mut first = true;
    for (from, to, diff) in diffs {
        if !first {
            writeln!(out, "")?;
        } else {
            first = false;
        }
        writeln!(out, "\t{}\t{}\tdifference ({}% CI)", from, to, significance)?;
        let from_stats = measurements.bench_stats(from);
        let to_stats = measurements.bench_stats(to);
        for (((metric, from), to), ci) in all_metrics()
            .zip(from_stats.iter())
            .zip(to_stats.iter())
            .zip(diff.iter())
        {
            writeln!(
                out,
                "{}\t{:.3} ± {:.3}\t{:.3} ± {:.3}\t{}",
                metric,
                from.mean(),
                from.sample_var().sqrt(),
                to.mean(),
                to.sample_var().sqrt(),
                fmt_ci(ci.interval(significance / 100.))
            )?;
        }
        writeln!(
            out,
            "samples\t{}\t{}",
            from_stats[0].count(),
            to_stats[0].count()
        )?;
    }

    Ok(String::from_utf8(out.into_inner()?)?)
}

fn fmt_ci((l, r): (f64, f64)) -> impl fmt::Display {
    let s = format!("[{:>+6.1}% .. {:>+6.1}%]", l, r);
    if l > 0. || r < 0. {
        Style::new().bold().paint(s)
    } else {
        Style::new().dimmed().paint(s)
    }
}
