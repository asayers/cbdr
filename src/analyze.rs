use crate::diff;
use crate::label::*;
use crate::pretty;
use crate::summarize;
use anyhow::*;
use std::time::*;
use structopt::*;

#[derive(StructOpt)]
pub struct Options {
    // /// The target CI width.  Applies to the 95% CI; units are percent of base.
    // #[structopt(long)]
    // threshold: Option<f64>,
    #[structopt(long)]
    deny_positive: bool,
    #[structopt(flatten)]
    diff_opts: diff::Options,
}

// The cbdr pipeline goes:
//
// sample -> summarize -> rate-limit -> diff -> pretty -> check-finished
//
// This subcommand just everything except sample
pub fn analyze(opts: Options) -> Result<()> {
    let mut rdr = csv::Reader::from_reader(std::io::stdin());
    let mut summarize = summarize::State::new();
    let stat_names = rdr
        .headers()
        .unwrap()
        .into_iter()
        .skip(1)
        .map(|x| x.to_string())
        .collect::<Vec<_>>();

    let mut pretty = pretty::State::new()?;
    let explicit_pairs = opts.diff_opts.pairs();
    macro_rules! print {
        () => {{
            let mut diff = if explicit_pairs.is_empty() {
                diff::State::new(summarize.guess_pairs().into_iter())
            } else {
                diff::State::new(explicit_pairs.iter().cloned())
            };
            diff.update(&summarize.all_measurements);
            pretty.print(&diff.diffs)?;
            diff
        }};
    }

    let mut last_print = Instant::now();
    for row in rdr.into_records() {
        let row = row?;
        let mut row = row.into_iter();
        let label = Label::from(row.next().unwrap().to_string());
        let values = row.map(|x| x.parse().unwrap());
        summarize.update(label, stat_names.iter().cloned().zip(values));

        if last_print.elapsed() > Duration::from_millis(100) {
            last_print = Instant::now();
            print!();

            // // Check to see if we're finished
            // if let Some(threshold) = opts.threshold {
            //     let worst = diff
            //         .diffs
            //         .iter()
            //         .flat_map(|diff| stats.iter().map(move |stat| *diff.cis.get(stat)?))
            //         .map(|x| x.map_or(std::f64::INFINITY, |x| x.r95_pc()))
            //         .fold(std::f64::NEG_INFINITY, f64::max);
            //     if worst < threshold {
            //         break;
            //     } else {
            //         info!("Threshold not reached: {}% > {}%", worst, threshold);
            //     }
            // }
        }
    }

    // Print the last set of diffs
    let diff = print!();

    if opts.deny_positive
        && diff
            .diffs
            .iter()
            .flat_map(|diff| diff.cis.values())
            .flatten()
            .any(|ci| ci.delta() > ci.r95)
    {
        bail!("Stat increased!");
    }

    Ok(())
}
