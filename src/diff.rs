use anyhow::*;
use confidence::*;
use log::*;
use serde::*;
use std::collections::{BTreeMap, HashMap};
use std::io::Write;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Options {
    comparisons: Vec<String>,
    #[structopt(short, long)]
    csv: bool,
    #[structopt(long)]
    elide_from: bool,
    #[structopt(long)]
    every_line: bool,
}

pub fn diff(opts: Options) -> Result<()> {
    let comparisons = opts
        .comparisons
        .iter()
        .flat_map(|x| {
            x.split(',')
                .zip(x.split(',').skip(1))
                .map(|(from, to)| (from.into(), to.into()))
        })
        .collect::<Vec<(String, String)>>();

    let mut rdr = csv::Reader::from_reader(std::io::stdin());
    let stat_names = rdr
        .headers()
        .unwrap()
        .into_iter()
        .skip(1)
        .map(|x| x.to_string())
        .collect::<Vec<_>>();
    let mut state = State::new(comparisons, stat_names);
    let mut stdout = std::io::stdout();
    for row in rdr.into_records() {
        let row = row?;
        let mut row = row.into_iter();
        let label = row.next().unwrap().to_string();
        state.update_measurements(&label, row.map(|x| x.parse().unwrap()));

        if opts.every_line {
            state.update_cis();
            state.print_json(&mut stdout)?;
        }
    }
    state.update_cis();
    if opts.csv {
        state.print_csv(stdout, opts.elide_from)?;
    } else {
        state.print_json(stdout)?;
    }
    Ok(())
}

#[derive(Clone, Debug)]
struct Measurements {
    count: usize,
    stats: Vec<rolling_stats::Stats<f64>>,
}
impl Measurements {
    fn new(n: usize) -> Measurements {
        Measurements {
            count: 0,
            stats: vec![rolling_stats::Stats::new(); n],
        }
    }
    fn iter<'a>(&'a self) -> impl Iterator<Item = Stats> + 'a {
        self.stats.iter().map(move |x| Stats {
            count: self.count,
            mean: x.mean,
            std_dev: x.std_dev,
        })
    }
}

struct State {
    comparisons: Vec<(String, String)>,
    stat_names: Vec<String>,
    measurements: HashMap<String, Measurements>,
    // Outer vec corresponds to comparison, inner to stat_name
    cis: Vec<Vec<Option<DiffCI>>>,
}

impl State {
    fn new(comparisons: Vec<(String, String)>, stat_names: Vec<String>) -> State {
        let mut cis = vec![];
        for _ in 0..comparisons.len() {
            cis.push(vec![None; stat_names.len()]);
        }
        State {
            measurements: HashMap::new(),
            cis,
            comparisons,
            stat_names,
        }
    }

    fn update_measurements(&mut self, label: &str, values: impl Iterator<Item = f64>) {
        let n = self.stat_names.len();
        let entry = self
            .measurements
            .entry(label.to_string())
            .or_insert_with(|| Measurements::new(n));
        entry.count += 1;
        for (stats, value) in entry.stats.iter_mut().zip(values) {
            stats.update(value);
        }
    }

    fn update_cis(&mut self) {
        for (i, (from, to)) in self.comparisons.iter_mut().enumerate() {
            if let Some(from) = self.measurements.get(from) {
                if let Some(to) = self.measurements.get(to) {
                    let new_cis = from.iter().zip(to.iter()).map(|(x, y)| {
                        let r = |sig_level| {
                            confidence_interval(sig_level, x, y)
                                .map_err(|e| match e {
                                    confidence::Error::NotEnoughData => (), // we expect some of these; ignore
                                    e => warn!("Skipping bad stats: {} ({:?} {:?})", e, x, y),
                                })
                                .ok()
                        };
                        Some(DiffCI {
                            mean_x: x.mean,
                            mean_y: y.mean,
                            r95: r(0.95)?,
                            r99: r(0.99)?,
                        })
                    });
                    self.cis[i].clear();
                    self.cis[i].extend(new_cis);
                }
            }
        }
    }

    fn output(&self) -> Vec<Diff> {
        self.comparisons
            .iter()
            .zip(self.cis.iter())
            .map(|(comp, cis)| Diff {
                from: comp.0.clone(),
                to: comp.1.clone(),
                cis: self
                    .stat_names
                    .iter()
                    .zip(cis.iter())
                    .map(|(k, ci)| (k.clone(), *ci))
                    .collect(),
            })
            .collect()
    }

    fn print_json(&self, mut stdout: impl Write) -> Result<()> {
        let s = serde_json::to_string(&self.output())?;
        writeln!(stdout, "{}", s)?;
        Ok(())
    }

    fn print_csv(
        &self,
        mut stdout: impl Write,
        elide_from: bool,
    ) -> Result<(), Box<std::io::Error>> {
        if elide_from {
            write!(stdout, "label")?;
        } else {
            write!(stdout, "from,to")?;
        }
        for x in &self.stat_names {
            write!(stdout, ",{}", x)?;
        }
        writeln!(stdout)?;
        for diff in self.output() {
            if elide_from {
                write!(stdout, "{}", diff.to)?;
            } else {
                write!(stdout, "{},{}", diff.from, diff.to)?;
            }
            for stat in &self.stat_names {
                if let Some(Some(ci)) = diff.cis.get(stat) {
                    write!(stdout, ",{} ± {}/{}", ci.mean_y - ci.mean_x, ci.r95, ci.r99)?;
                } else {
                    write!(stdout, ",insufficient data")?;
                }
            }
            writeln!(stdout)?;
        }
        Ok(())
    }
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
}
