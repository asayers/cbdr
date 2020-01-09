use crate::label::*;
use crate::pretty;
use crate::summarize;
use anyhow::*;
use log::*;
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
    /// Benchs to compare.  If "base" is not specified, they'll be compared
    /// consecutively.
    pub labels: Vec<String>,
}
impl Options {
    pub fn labels_in_order<'a>(&'a self) -> Box<dyn Iterator<Item = Bench> + 'a> {
        if self.labels.is_empty() {
            Box::new(all_benches())
        } else {
            Box::new(self.labels.iter().map(|x| Bench::from(x.as_str())))
        }
    }
    pub fn pairs<'a>(&'a self) -> Box<dyn Iterator<Item = (Bench, Bench)> + 'a> {
        if let Some(base) = &self.base {
            let base = Bench::from(base.as_str());
            Box::new(
                self.labels_in_order()
                    .filter(move |x| *x != base)
                    .map(move |x| (base, x)),
            )
        } else {
            Box::new(self.labels_in_order().zip(self.labels_in_order().skip(1)))
        }
    }
}

// summarize -> rate-limit -> diff -> pretty print
pub fn analyze(opts: Options) -> Result<()> {
    let mut rdr = csv::Reader::from_reader(std::io::stdin());
    let mut headers = rdr.headers().unwrap().into_iter();
    let first = headers.next().unwrap();
    info!("Assuming \"{}\" column is the benchmark name", first);
    init_metrics(headers.map(|x| x.to_string()).collect());
    let mut measurements = summarize::Measurements::default();

    let mut printer = Printer::new()?;

    let mut last_print = Instant::now();
    for row in rdr.into_records() {
        let row = row?;
        let mut row = row.into_iter();
        let bench = Bench::from(row.next().unwrap());
        let values = row.map(|x| x.parse().unwrap());
        measurements.update(bench, values);

        if last_print.elapsed() > Duration::from_millis(100) {
            last_print = Instant::now();
            let diffs = opts.pairs().map(|(from, to)| {
                let diff = measurements.diff(from, to);
                (from, to, diff)
            });
            let out = pretty::render(&measurements, diffs)?;
            printer.print(out)?;

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
    let diffs = opts.pairs().map(|(from, to)| {
        let diff = measurements.diff(from, to);
        (from, to, diff)
    });
    let out = pretty::render(&measurements, diffs)?;
    printer.print(out)?;

    if opts.deny_positive {
        for (from, to) in opts.pairs() {
            for (idx, ci) in measurements.diff(from, to).0.into_iter().enumerate() {
                let metric = Metric(idx);
                if ci.delta() > ci.ci(0.95) {
                    bail!("{}..{}: {} increased!", from, to, metric);
                }
            }
        }
    }

    Ok(())
}

pub struct Printer {
    stdout: Box<term::StdoutTerminal>,
    /// The number of lines output in the previous iteration
    n: usize,
}
impl Printer {
    pub fn new() -> Result<Printer> {
        Ok(Printer {
            stdout: term::stdout().ok_or_else(|| anyhow!("Couldn't open stdout as a terminal"))?,
            n: 0,
        })
    }
    // Clear the previous output and replace it with the new output
    pub fn print(&mut self, out: Vec<u8>) -> Result<()> {
        for _ in 0..self.n {
            self.stdout.cursor_up()?;
            self.stdout.delete_line()?;
        }
        self.stdout.write_all(&out)?;
        self.n = out.into_iter().filter(|c| *c == b'\n').count();
        Ok(())
    }
}
