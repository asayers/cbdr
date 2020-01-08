use crate::label::*;
use crate::pretty;
use crate::summarize;
use anyhow::*;
use std::collections::BTreeMap;
use std::time::*;
use structopt::*;

#[derive(StructOpt)]
pub struct Options {
    // /// The target CI width.  Applies to the 95% CI; units are percent of base.
    // #[structopt(long)]
    // threshold: Option<f64>,
    #[structopt(long)]
    deny_positive: bool,
    /// A "base" label.  If specified, all labels will be compared to this.
    #[structopt(long)]
    pub base: Option<String>,
    /// Labels to compare.  If "base" is not specified, they'll be compared
    /// consecutively.
    pub labels: Vec<String>,
}
impl Options {
    pub fn pairs(&self) -> Vec<(Label, Label)> {
        if let Some(base) = &self.base {
            let base = Label::from(base.clone());
            self.labels
                .iter()
                .cloned()
                .map(Label::from)
                .map(move |to| (base.clone(), to))
                .collect()
        } else {
            let iter = self.labels.iter().cloned().map(Label::from);
            iter.clone().zip(iter.skip(1)).collect()
        }
    }
}

// summarize -> rate-limit -> diff -> pretty print
pub fn analyze(opts: Options) -> Result<()> {
    let mut rdr = csv::Reader::from_reader(std::io::stdin());
    let mut measurements = summarize::Measurements::default();
    let stat_names = rdr
        .headers()
        .unwrap()
        .into_iter()
        .skip(1)
        .map(|x| x.to_string())
        .collect::<Vec<_>>();

    let mut pretty = pretty::State::new()?;
    let explicit_pairs = opts.pairs();
    macro_rules! print {
        () => {{
            let pairs = if explicit_pairs.is_empty() {
                measurements.guess_pairs()
            } else {
                explicit_pairs.clone()
            };
            let mut diffs = BTreeMap::default();
            for (from, to) in pairs {
                let diff = measurements.diff(from.clone(), to.clone());
                diffs.insert((from, to), diff);
            }
            pretty.print(&stat_names, &measurements, &diffs)?;
            diffs
        }};
    }

    let mut last_print = Instant::now();
    for row in rdr.into_records() {
        let row = row?;
        let mut row = row.into_iter();
        let label = Label::from(row.next().unwrap().to_string());
        let values = row.map(|x| x.parse().unwrap());
        measurements.update(label, stat_names.iter().cloned().zip(values));

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
    let diffs = print!();

    if opts.deny_positive {
        for ((from, to), diff) in diffs {
            for (stat_name, ci) in diff.0 {
                if let Some(ci) = ci {
                    if ci.delta() > ci.ci(0.95) {
                        bail!("{}..{}: {} increased!", from, to, stat_name);
                    }
                }
            }
        }
    }

    Ok(())
}
