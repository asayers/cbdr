use crate::diff::*;
use crate::label::*;
use crate::summarize::*;
use ansi_term::Style;
use anyhow::*;
use std::fmt;
use std::io::Write;

pub fn render(
    measurements: &Measurements,
    diffs: impl Iterator<Item = (Bench, Bench, Diff)>,
) -> Result<Vec<u8>> {
    let mut out = tabwriter::TabWriter::new(Vec::<u8>::new());

    // Print the summary table
    write!(out, "benchmark\tsamples")?;
    for metric in all_metrics() {
        write!(out, "\t{}", metric)?;
    }
    writeln!(out)?;
    for bench in all_benches() {
        let count = measurements
            .iter_label(bench)
            .map(|x| (x.1).0)
            .next()
            .unwrap_or(0);
        write!(out, "{}\t{}", bench, count)?;
        for metric in all_metrics() {
            write!(
                out,
                "\t{:.3}",
                measurements
                    .get(bench, metric)
                    .map(|stats| stats.1.mean)
                    .unwrap_or(std::f64::NAN)
            )?;
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
        for (idx, ci) in diff.0.iter().enumerate() {
            let metric = Metric(idx);
            writeln!(
                out,
                "    {}\t{}\t{}\t{}\t{}",
                metric,
                PrettyCI(*ci, 0.95),
                PrettyCI(*ci, 0.99),
                PrettyCI(*ci, 0.999),
                PrettyCI(*ci, 0.9999),
            )?;
        }
    }

    let out = out.into_inner()?;
    Ok(out)
}

struct PrettyCI(DiffCI, f64);

impl fmt::Display for PrettyCI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let delta = self.0.stats_y.mean - self.0.stats_x.mean;
        let width = self.0.ci(self.1);
        let left = format!("{:+.1}", 100. * (delta - width) / self.0.stats_x.mean);
        let right = format!("{:+.1}", 100. * (delta + width) / self.0.stats_x.mean);
        let style = if delta > width || delta < -width {
            Style::new().bold()
        } else {
            Style::new().dimmed()
        };
        let s = format!("[{:>6}% .. {:>6}%]", left, right);
        write!(f, "{}", style.paint(s))
    }
}
