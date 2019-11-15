use crate::diff::*;
use ansi_term::{Color, Style};
use anyhow::*;
use std::fmt;
use std::io::{stdin, stdout, BufRead, BufReader, Write};

pub fn pretty() -> Result<()> {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    for diff in BufReader::new(stdin()).lines() {
        let diff: Diff = serde_json::from_str(&diff?)?;
        writeln!(stdout, "{}..{}:", diff.from, diff.to)?;
        let n = diff.cis.len();
        let mut out = tabwriter::TabWriter::new(&mut stdout);
        for (stat, ci) in diff.cis {
            writeln!(out, "\t{}:\t{}", stat, PrettyCI(ci, n))?;
        }
        out.flush()?;
        writeln!(stdout, "    (CIs shown at the {}% significance level)", diff.significance_level* 100.)?;
    }
    Ok(())
}

// Always takes 21 characters
pub struct PrettyCI(
    pub Option<(f64, f64)>,
    pub usize, /* total number of CIs */
);

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
            let center = ci.0 * scale;
            let radius = ci.1 * scale;
            let critical = ci.1 * self.1 as f64;
            if ci.0 - ci.1 < 0. && 0. < ci.0 + ci.1 {
                let center = format!("{:.3}{}", center, suffix);
                let radius = format!("{:.3}{}", radius, suffix);
                write!(f, "{:>9} ± {:<9}", center, radius)
            } else if ci.0 - critical < 0. && 0. < ci.0 + critical {
                let center = format!("{:.3}{}", center, suffix);
                let radius = format!("{:.3}{}", radius, suffix);
                write!(
                    f,
                    "{}{:>9} ± {:<9}{}",
                    Color::Yellow.prefix(),
                    center,
                    radius,
                    Color::Yellow.suffix()
                )
            } else {
                let center = format!("{:.3}{}", center, suffix);
                let radius = format!("{:.3}{}", radius, suffix);
                write!(
                    f,
                    "{}{:>9} ± {:<9}{}",
                    Color::Red.prefix(),
                    center,
                    radius,
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
