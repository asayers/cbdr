use crate::label::*;
use crate::summarize::*;
use anyhow::*;
use confidence::*;
use log::*;
use serde::*;
use std::collections::{BTreeMap, BTreeSet};
use std::io::{BufRead, BufReader, Write};
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Options {
    /// A "base" label.  If specified, all labels will be compared to this.
    #[structopt(long)]
    pub base: Option<String>,
    /// Labels to compare.  If "base" is not specified, they'll be compared
    /// consecutively.
    pub labels: Vec<String>,
}
impl Options {
    pub fn all_labels(self) -> Vec<Label> {
        self.base
            .into_iter()
            .chain(self.labels.into_iter())
            .map(Label::from)
            .collect()
    }
    pub fn pairs(&self) -> Box<dyn Iterator<Item = (Label, Label)> + '_> {
        if let Some(base) = &self.base {
            let base = Label::from(base.clone());
            Box::new(
                self.labels
                    .iter()
                    .cloned()
                    .map(Label::from)
                    .map(move |to| (base.clone(), to)),
            )
        } else {
            let iter = self.labels.iter().cloned().map(Label::from);
            Box::new(iter.clone().zip(iter.skip(1)))
        }
    }
}

pub struct State {
    pub diffs: Vec<Diff>,
}
impl State {
    pub fn new(pairs: impl Iterator<Item = (Label, Label)>) -> State {
        State {
            diffs: pairs
                .map(|(from, to)| {
                    let cis = BTreeMap::new();
                    Diff { from, to, cis }
                })
                .collect::<Vec<_>>(),
        }
    }
    pub fn update(&mut self, measurements: &BTreeMap<Label, Measurements>) {
        for diff in self.diffs.iter_mut() {
            if let Some(from) = measurements.get(&diff.from) {
                if let Some(to) = measurements.get(&diff.to) {
                    diff.cis = diff_ci(from, to);
                }
            }
        }
    }
}

pub fn diff(opts: Options) -> Result<()> {
    let mut state = State::new(opts.pairs());
    for line in BufReader::new(std::io::stdin()).lines() {
        let measurements: BTreeMap<Label, Measurements> = serde_json::from_str(&line?)?;
        state.update(&measurements);
        let s = serde_json::to_string(&state.diffs)?;
        writeln!(std::io::stdout(), "{}", s)?;
    }
    Ok(())
}

fn ci(sig_level: f64, x: Stats, y: Stats) -> Option<f64> {
    confidence_interval(sig_level, x, y)
        .map_err(|e| match e {
            confidence::Error::NotEnoughData => (), // we expect some of these; ignore
            e => warn!("Skipping bad stats: {} ({:?} {:?})", e, x, y),
        })
        .ok()
}

#[derive(Serialize, Debug, Clone, PartialEq, Deserialize)]
pub struct Diff {
    pub from: Label,
    pub to: Label,
    #[serde(flatten)]
    pub cis: BTreeMap<String, Option<DiffCI>>,
}
#[derive(Serialize, Debug, Clone, PartialEq, Deserialize, Copy)]
pub struct DiffCI {
    pub mean_x: f64,
    pub mean_y: f64,
    pub r95: f64,
    pub r99: f64,
}
impl DiffCI {
    pub fn delta(self) -> f64 {
        self.mean_y - self.mean_x
    }
    pub fn delta_pc(self) -> f64 {
        100. * self.delta() / self.mean_x
    }
    pub fn r95_pc(self) -> f64 {
        100. * self.r95 / self.mean_x
    }
    pub fn r99_pc(self) -> f64 {
        100. * self.r99 / self.mean_x
    }
}
fn diff_ci(xs: &Measurements, ys: &Measurements) -> BTreeMap<String, Option<DiffCI>> {
    let keys = xs.0.keys().chain(ys.0.keys()).collect::<BTreeSet<_>>();
    keys.into_iter()
        .map(|stat| {
            let ci = (|| {
                let x = xs.get(stat)?;
                let y = ys.get(stat)?;
                Some(DiffCI {
                    mean_x: x.mean,
                    mean_y: y.mean,
                    r95: ci(0.95, x, y)?,
                    r99: ci(0.99, x, y)?,
                })
            })();
            (stat.clone(), ci)
        })
        .collect()
}
