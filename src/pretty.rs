use crate::diff::*;
use crate::label::*;
use crate::summarize::*;
use ansi_term::Style;
use anyhow::*;
use std::fmt;
use std::io::Write;

pub struct State {
    stdout: Box<term::StdoutTerminal>,
    /// The number of lines output in the previous iteration
    n: usize,
}
impl State {
    pub fn new() -> Result<State> {
        let stdout = term::stdout().ok_or_else(|| anyhow!("Couldn't open stdout as a terminal"))?;
        let n = 0;
        Ok(State { stdout, n })
    }
    pub fn print(
        &mut self,
        all_metrics: &[Metric],
        measurements: &Measurements,
        diffs: &[(Label, Label, Diff)],
    ) -> Result<()> {
        // Clear the previous output
        for _ in 0..self.n {
            self.stdout.cursor_up()?;
            self.stdout.delete_line()?;
        }
        self.n = 0;

        {
            let mut out = tabwriter::TabWriter::new(&mut self.stdout);
            write!(out, "benchmark\tsamples")?;
            for metric in all_metrics {
                write!(out, "\t{}", metric)?;
            }
            writeln!(out)?;
            self.n += 1;
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
                            .stats
                            .get(&(label, *metric))
                            .map(|stats| stats.1.mean)
                            .unwrap_or(std::f64::NAN)
                    )?;
                }
                writeln!(out)?;
                self.n += 1;
            }
            out.flush()?;
        }

        for (from, to, diff) in diffs {
            self.n += diff.0.len() + 4;
            writeln!(self.stdout, "\n{} vs {}:\n", from, to)?;
            let mut out = tabwriter::TabWriter::new(&mut self.stdout);
            write_key(&mut out)?;
            for (stat, ci) in &diff.0 {
                writeln!(out, "    {}\t{}", stat, PrettyCI(*ci))?;
            }
            out.flush()?;
        }
        Ok(())
    }
}

fn write_key(mut out: impl Write) -> Result<()> {
    writeln!(out, "\t ratio\t        95% CI\t        99% CI",)?;
    Ok(())
}

struct PrettyCI(Option<DiffCI>);

macro_rules! highlight_if {
    ($cond: expr, $x:expr) => {
        if $cond {
            $x.bold()
        } else {
            $x.dimmed()
        }
    };
}
impl fmt::Display for PrettyCI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ci) = self.0 {
            let delta = ci.stats_y.mean - ci.stats_x.mean;
            // let center = format!("{:+.1}%", 100. * delta / ci.stats_x.mean);
            let l95 = format!("{:+.1}", 100. * (delta - ci.ci(0.95)) / ci.stats_x.mean);
            let r95 = format!("{:+.1}", 100. * (delta + ci.ci(0.95)) / ci.stats_x.mean);
            let l99 = format!("{:+.1}", 100. * (delta - ci.ci(0.99)) / ci.stats_x.mean);
            let r99 = format!("{:+.1}", 100. * (delta + ci.ci(0.99)) / ci.stats_x.mean);
            let s95 = Style::new(); // .fg(Color::Yellow);
            let s99 = Style::new(); // .fg(Color::Red);
            let s95 = highlight_if!(delta > ci.ci(0.95) || delta < -ci.ci(0.95), s95);
            let s99 = highlight_if!(delta > ci.ci(0.99) || delta < -ci.ci(0.99), s99);
            write!(
                f,
                "{}\t{}\t{}", // \tΔ_99% = [{:>8} ⋯ {:<8}]\t",
                Style::new().paint(format!("{:.3}x", ci.stats_y.mean / ci.stats_x.mean)),
                s95.paint(format!("[{:>6}% .. {:>6}%]", l95, r95)),
                s99.paint(format!("[{:>6}% .. {:>6}%]", l99, r99)),
            )
        } else {
            write!(
                f,
                "{}",
                Style::new().dimmed().paint("  insufficient data  ")
            )
        }
    }
}
