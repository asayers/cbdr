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
    comparisons: Vec<String>,
}
impl Options {
    fn pairs(&self) -> impl Iterator<Item = (&str, &str)> + '_ {
        self.comparisons
            .iter()
            .flat_map(|x| x.split(',').zip(x.split(',').skip(1)))
    }
}

pub fn diff(opts: Options) -> Result<()> {
    let mut state = opts
        .pairs()
        .map(|(from, to)| Diff {
            from: from.into(),
            to: to.into(),
            cis: BTreeMap::new(),
        })
        .collect::<Vec<_>>();
    for line in BufReader::new(std::io::stdin()).lines() {
        let measurements: BTreeMap<String, Measurements> = serde_json::from_str(&line?)?;
        for diff in state.iter_mut() {
            if let Some(from) = measurements.get(&diff.from) {
                if let Some(to) = measurements.get(&diff.to) {
                    diff.cis = diff_ci(from, to);
                }
            }
        }
        let s = serde_json::to_string(&state)?;
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
    pub from: String,
    pub to: String,
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
    let keys = xs
        .stats
        .keys()
        .chain(ys.stats.keys())
        .collect::<BTreeSet<_>>();
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
