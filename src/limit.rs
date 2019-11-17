use crate::diff::*;
use anyhow::*;
use log::*;
use std::io::{stdin, stdout, BufRead, BufReader, Write};
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Options {
    threshold: f64,
}

pub fn limit(opts: Options) -> Result<()> {
    let stdout = stdout();
    let mut stdout = stdout.lock();
    for line in BufReader::new(stdin()).lines() {
        let diffs: Vec<Diff> = serde_json::from_str(&line?)?;
        let s = serde_json::to_string(&diffs)?;
        writeln!(stdout, "{}", s)?;

        let worst = diffs
            .iter()
            .flat_map(|diff| diff.cis.values())
            .map(|ci| ci.map_or(std::f64::INFINITY, |x| x.r95_pc()))
            .fold(std::f64::NEG_INFINITY, f64::max);
        if worst < opts.threshold {
            break;
        } else {
            info!("Threshold not reached: {}% > {}%", worst, opts.threshold);
        }
    }
    Ok(())
}
