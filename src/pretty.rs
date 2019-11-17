use crate::diff::*;
use ansi_term::{Color, Style};
use anyhow::*;
use std::fmt;
use std::io::{stdin, BufRead, BufReader, Write};

pub fn pretty() -> Result<()> {
    let mut stdout = term::stdout().ok_or(anyhow!("Couldn't open stdout as a terminal"))?;
    let mut n = 0; // The number of lines output in the previous iteration
    for line in BufReader::new(stdin()).lines() {
        let diffs: Vec<Diff> = serde_json::from_str(&line?)?;
        for diff in diffs {
            // Clear the previous output
            for _ in 0..n {
                stdout.cursor_up()?;
                stdout.delete_line()?;
            }
            n = diff.cis.len() + 1;
            writeln!(stdout, "{}..{}:", diff.from, diff.to)?;
            let mut out = tabwriter::TabWriter::new(&mut stdout);
            for (stat, ci) in diff.cis {
                writeln!(out, "\t{}:\t{}", stat, PrettyCI(ci))?;
            }
            out.flush()?;
        }
    }
    Ok(())
}

// Always takes 21 characters
pub struct PrettyCI(pub Option<DiffCI>);

impl fmt::Display for PrettyCI {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ci) = self.0 {
            let center = format!("{:.3}%", ci.delta_pc());
            let r95 = format!("{:.3}% (95%)", ci.r95_pc());
            let r99 = format!("{:.3}% (99%)", ci.r99_pc());
            if ci.delta() - ci.r95 < 0. && 0. < ci.delta() + ci.r95 {
                write!(f, "{:>9} ± {:>13}, {:>13}", center, r95, r99)
            } else if ci.delta() - ci.r99 < 0. && 0. < ci.delta() + ci.r99 {
                write!(
                    f,
                    "{}{:>9} ± {:>13}, ± {:>13}{}",
                    Color::Yellow.prefix(),
                    center,
                    r95,
                    r99,
                    Color::Yellow.suffix()
                )
            } else {
                write!(
                    f,
                    "{}{:>9} ± {:>13}, {:>13}{}",
                    Color::Red.prefix(),
                    center,
                    r95,
                    r99,
                    Color::Red.suffix()
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
