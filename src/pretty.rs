use crate::diff::*;
use crate::label::*;
use crate::summarize::*;
use ansi_term::Style;
use anyhow::*;
use std::fmt;
use std::io::Write;

pub fn render(
    all_metrics: &[Metric],
    measurements: &Measurements,
    diffs: &[(Bench, Bench, Diff)],
) -> Result<Vec<u8>> {
    let mut out = tabwriter::TabWriter::new(Vec::<u8>::new());

    // Print the summary table
    write!(out, "benchmark\tsamples")?;
    for metric in all_metrics {
        write!(out, "\t{}", metric)?;
    }
    writeln!(out)?;
    for label in measurements.labels() {
        let count = measurements
            .iter_label(label)
            .map(|x| (x.1).0)
            .next()
            .unwrap_or(0);
        write!(out, "{}\t{}", label, count)?;
        for metric in all_metrics {
            write!(
                out,
                "\t{:.3}",
                measurements
                    .get(label, *metric)
                    .map(|stats| stats.1.mean)
                    .unwrap_or(std::f64::NAN)
            )?;
        }
        writeln!(out)?;
    }

    // Print the diff tables
    for (from, to, diff) in diffs {
        writeln!(out, "\n{} vs {}:\n", from, to)?;
        write_key(&mut out)?;
        for (idx, ci) in diff.0.iter().enumerate() {
            let metric = Metric(idx);
            writeln!(out, "    {}\t{}", metric, PrettyCI(*ci))?;
        }
    }

    let out = out.into_inner()?;
    Ok(out)
}

fn write_key(mut out: impl Write) -> Result<()> {
    writeln!(out, "\t        95% CI\t        99% CI\t ratio",)?;
    Ok(())
}

struct PrettyCI(DiffCI);

macro_rules! highlight_if {
    ($cond: expr) => {
        if $cond {
            Style::new().bold()
        } else {
            Style::new().dimmed()
        }
    };
}
impl fmt::Display for PrettyCI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let ci = self.0;
        let delta = ci.stats_y.mean - ci.stats_x.mean;
        let l95 = format!("{:+.1}", 100. * (delta - ci.ci(0.95)) / ci.stats_x.mean);
        let r95 = format!("{:+.1}", 100. * (delta + ci.ci(0.95)) / ci.stats_x.mean);
        let l99 = format!("{:+.1}", 100. * (delta - ci.ci(0.99)) / ci.stats_x.mean);
        let r99 = format!("{:+.1}", 100. * (delta + ci.ci(0.99)) / ci.stats_x.mean);
        let s95 = highlight_if!(delta > ci.ci(0.95) || delta < -ci.ci(0.95));
        let s99 = highlight_if!(delta > ci.ci(0.99) || delta < -ci.ci(0.99));
        write!(
            f,
            "{}\t{}\t{:.3}x",
            s95.paint(format!("[{:>6}% .. {:>6}%]", l95, r95)),
            s99.paint(format!("[{:>6}% .. {:>6}%]", l99, r99)),
            ci.stats_y.mean / ci.stats_x.mean,
        )
    }
}
