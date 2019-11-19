use crate::diff::*;
use ansi_term::{Color, Style};
use anyhow::*;
use std::fmt;
use std::io::{stdin, BufRead, BufReader, Write};

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
    pub fn print(&mut self, diffs: &[Diff]) -> Result<()> {
        // Clear the previous output
        for _ in 0..self.n {
            self.stdout.cursor_up()?;
            self.stdout.delete_line()?;
        }
        self.n = 0;
        for diff in diffs {
            self.n += diff.cis.len() + 3;
            writeln!(self.stdout, "\n{}..{}:", diff.from, diff.to)?;
            let mut out = tabwriter::TabWriter::new(&mut self.stdout);
            writeln!(out, "\t\t{}", write_key())?;
            for (stat, ci) in &diff.cis {
                writeln!(out, "\t{}:\t{}", stat, PrettyCI(*ci))?;
            }
            out.flush()?;
        }
        Ok(())
    }
}

pub fn pretty() -> Result<()> {
    let mut state = State::new()?;
    for line in BufReader::new(stdin()).lines() {
        let diffs: Vec<Diff> = serde_json::from_str(&line?)?;
        state.print(&diffs)?;
    }
    Ok(())
}

fn write_key() -> String {
    let style_95 = Style::new().fg(Color::Yellow);
    let style_99 = Style::new().fg(Color::Red);
    format!(
        "  {} {} {} {} {}",
        style_99.paint(format!("{:>9}", "-99%")),
        style_95.paint(format!("{:>9}", "-95%")),
        format!("{:>9}", "Δ"),
        style_95.paint(format!("{:>9}", "+95%")),
        style_99.paint(format!("{:>9}", "+99%")),
    )
}

struct PrettyCI(Option<DiffCI>);

impl fmt::Display for PrettyCI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ci) = self.0 {
            let delta = ci.mean_y - ci.mean_x;
            let center = format!("{:+.3}%", 100. * delta / ci.mean_x);
            let l95 = format!("{:+.3}%", 100. * (delta - ci.r95) / ci.mean_x);
            let r95 = format!("{:+.3}%", 100. * (delta + ci.r95) / ci.mean_x);
            let l99 = format!("{:+.3}%", 100. * (delta - ci.r99) / ci.mean_x);
            let r99 = format!("{:+.3}%", 100. * (delta + ci.r99) / ci.mean_x);
            let s95 = Style::new(); // .fg(Color::Yellow);
            let s99 = Style::new(); // .fg(Color::Red);
            let sl95 = if delta > ci.r95 { s95.bold() } else { s95.dimmed() };
            let sr95 = if delta < -ci.r95 { s95.bold() } else { s95.dimmed() };
            let sl99 = if delta > ci.r99 { s99.bold() } else { s99.dimmed() };
            let sr99 = if delta < -ci.r99 { s99.bold() } else { s99.dimmed() };
            write!(
                f,
                "[ {} {} {} {} {} ]  {}", // \tΔ_99% = [{:>8} ⋯ {:<8}]\t",
                sl99.paint(format!("{:>9}", l99)),
                sl95.paint(format!("{:>9}", l95)),
                format!("{:>9}", center),
                sr95.paint(format!("{:>9}", r95)),
                sr99.paint(format!("{:>9}", r99)),
                Style::new()
                    .dimmed()
                    .paint(format!("({:.3} -> {:.3})", ci.mean_x, ci.mean_y)),
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

struct PrettyCI2(Option<DiffCI>);
impl fmt::Display for PrettyCI2 {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ci) = self.0 {
            let center = format!("{:+.3}%", ci.delta_pc());
            let r95 = format!("{:.3}% (95%)", ci.r95_pc());
            let r99 = format!("{:.3}% (99%)", ci.r99_pc());
            if ci.delta() - ci.r95 < 0. && 0. < ci.delta() + ci.r95 {
                write!(f, "{:>9} ± {:>13}, ± {:>13}", center, r95, r99)
            } else if ci.delta() - ci.r99 < 0. && 0. < ci.delta() + ci.r99 {
                write!(
                    f,
                    "{}{:>9}{} ± {:>13}, ± {:>13}",
                    Color::Yellow.prefix(),
                    center,
                    Color::Yellow.suffix(),
                    r95,
                    r99,
                )
            } else {
                write!(
                    f,
                    "{}{:>9}{} ± {:>13}, ± {:>13}",
                    Color::Red.prefix(),
                    center,
                    Color::Red.suffix(),
                    r95,
                    r99,
                )
            }
        } else {
            write!(
                f,
                "{}",
                Style::new().dimmed().paint("  insufficient data  ")
            )
        }
    }
}
