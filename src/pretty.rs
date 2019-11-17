use crate::diff::*;
use ansi_term::{Color, Style};
use anyhow::*;
use std::fmt;
use std::io::{stdin, stdout, BufRead, BufReader, Write};

pub fn pretty() -> Result<()> {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    for line in BufReader::new(stdin()).lines() {
        let diffs: Vec<Diff> = serde_json::from_str(&line?)?;
        for diff in diffs {
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
            let (scale, suffix) = match ci.0 {
                x if x.abs() < 0.0001 => (1_000_000., "u"),
                x if x.abs() < 0.1 => (1_000., "m"),
                x if x.abs() >= 1_000. => (0.001, "k"),
                x if x.abs() >= 1_000_000. => (0.000001, "M"),
                _ => (1., ""),
            };
            let center = format!("{:.3}{}", ci.delta() * scale, suffix);
            let r95 = format!("{:.3}{}", ci.r95 * scale, suffix);
            let r99 = format!("{:.3}{}", ci.r99 * scale, suffix);
            let critical = ci.r95 * self.1 as f64;
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
