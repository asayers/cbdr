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
        Ok(State {
            stdout: term::stdout().ok_or(anyhow!("Couldn't open stdout as a terminal"))?,
            n: 0,
        })
    }
    pub fn print(&mut self, diffs: &[Diff]) -> Result<()> {
        // Clear the previous output
        for _ in 0..self.n {
            self.stdout.cursor_up()?;
            self.stdout.delete_line()?;
        }
        self.n = 0;
        for diff in diffs {
            self.n += diff.cis.len() + 2;
            writeln!(self.stdout, "\n{}..{}:", diff.from, diff.to)?;
            let mut out = tabwriter::TabWriter::new(&mut self.stdout);
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

// Always takes 21 characters
pub struct PrettyCI(pub Option<DiffCI>);

impl fmt::Display for PrettyCI {
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
