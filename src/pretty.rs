use crate::diff::*;
use crate::label::*;
use crate::summarize::*;
use ansi_term::{Color, Style};
use anyhow::*;
use std::collections::BTreeMap;
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
        stat_names: &[String],
        measurements: &Measurements,
        diffs: &BTreeMap<(Label, Label), Diff>,
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
            for stat_name in stat_names {
                write!(out, "\t{}", stat_name.clone())?;
            }
            writeln!(out)?;
            self.n += 1;
            for (idx, label) in diffs
                .iter()
                .enumerate()
                .take(1)
                .map(|(idx, ((label, _), _))| (idx, label))
                .chain(
                    diffs
                        .iter()
                        .enumerate()
                        .map(|(idx, ((_, label), _))| (idx + 1, label)),
                )
            {
                let count = measurements
                    .iter_label(label.clone())
                    .map(|x| (x.1).0)
                    .next()
                    .unwrap_or(0);
                write!(
                    out,
                    "{}\t{}",
                    Style::new().fg(idx_to_color(idx)).paint(&label.0),
                    count,
                )?;
                for stat_name in stat_names {
                    write!(
                        out,
                        "\t{:.3}",
                        measurements
                            .0
                            .get(&(label.clone(), stat_name.clone()))
                            .map(|stats| stats.1.mean)
                            .unwrap_or(std::f64::NAN)
                    )?;
                }
                writeln!(out)?;
                self.n += 1;
            }
            out.flush()?;
        }

        for (idx, ((from, to), diff)) in diffs.into_iter().enumerate() {
            self.n += diff.0.len() + 4;
            let from_color = idx_to_color(idx);
            let to_color = idx_to_color(idx + 1);
            writeln!(
                self.stdout,
                "\n{}{}{}:\n",
                Style::new().fg(from_color).paint(&from.0),
                Style::new().paint(" vs "),
                Style::new().fg(to_color).paint(&to.0)
            )?;
            let mut out = tabwriter::TabWriter::new(&mut self.stdout);
            write_key(&mut out)?;
            for (stat, ci) in &diff.0 {
                writeln!(out, "    {}\t{}", stat, PrettyCI(*ci, from_color, to_color))?;
            }
            out.flush()?;
        }
        Ok(())
    }
}

fn idx_to_color(idx: usize) -> Color {
    match idx % 4 {
        0 => Color::Purple,
        1 => Color::Yellow,
        2 => Color::Cyan,
        3 => Color::Green,
        _ => unreachable!(),
    }
}

fn write_key(mut out: impl Write) -> Result<()> {
    writeln!(out, "\t ratio\t        95% CI\t        99% CI",)?;
    Ok(())
}

struct PrettyCI(Option<DiffCI>, Color, Color);

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
