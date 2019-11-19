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
            write_key(&mut out)?;
            for (stat, ci) in &diff.cis {
                writeln!(out, "\t{}\t{}", stat, PrettyCI(*ci))?;
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

fn write_key(mut out: impl Write) -> Result<()> {
    let style_95 = Style::new().fg(Color::Yellow);
    let style_99 = Style::new().fg(Color::Red);
    writeln!(
        out,
        "\t\t{}\t{}\tΔ\t{}\t{}\tbefore\tafter\tratio",
        style_99.paint(format!("{}", "-99%")),
        style_95.paint(format!("{}", "-95%")),
        style_95.paint(format!("{}", "+95%")),
        style_99.paint(format!("{}", "+99%")),
    )?;
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
            let delta = ci.mean_y - ci.mean_x;
            let center = format!("{:+.3}%", 100. * delta / ci.mean_x);
            let l95 = format!("{:+.3}%", 100. * (delta - ci.r95) / ci.mean_x);
            let r95 = format!("{:+.3}%", 100. * (delta + ci.r95) / ci.mean_x);
            let l99 = format!("{:+.3}%", 100. * (delta - ci.r99) / ci.mean_x);
            let r99 = format!("{:+.3}%", 100. * (delta + ci.r99) / ci.mean_x);
            let s95 = Style::new(); // .fg(Color::Yellow);
            let s99 = Style::new(); // .fg(Color::Red);
            let sl95 = highlight_if!(delta > ci.r95, s95);
            let sr95 = highlight_if!(delta < -ci.r95, s95);
            let sl99 = highlight_if!(delta > ci.r99, s99);
            let sr99 = highlight_if!(delta < -ci.r99, s99);
            write!(
                f,
                "{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}", // \tΔ_99% = [{:>8} ⋯ {:<8}]\t",
                sl99.paint(format!("{}", l99)),
                sl95.paint(format!("{}", l95)),
                format!("{}", center),
                sr95.paint(format!("{}", r95)),
                sr99.paint(format!("{}", r99)),
                Style::new().dimmed().paint(format!("{:.3}", ci.mean_x)),
                Style::new().dimmed().paint(format!("{:.3}", ci.mean_y)),
                Style::new()
                    .dimmed()
                    .paint(format!("{:.3}", ci.mean_y / ci.mean_x)),
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
