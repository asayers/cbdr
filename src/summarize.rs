use anyhow::*;
use confidence::*;
use serde::*;
use std::collections::BTreeMap;
use std::io::Write;
use structopt::StructOpt;

#[derive(StructOpt)]
pub struct Options {
    #[structopt(long, short)]
    rate_limit: Option<f64>,
}

pub fn summarize(opts: Options) -> Result<()> {
    let mut rdr = csv::Reader::from_reader(std::io::stdin());
    let stat_names = rdr
        .headers()
        .unwrap()
        .into_iter()
        .skip(1)
        .map(|x| x.to_string())
        .collect::<Vec<_>>();
    let mut all_measurements = BTreeMap::new();
    let mut stdout = std::io::stdout();
    for row in rdr.into_records() {
        let row = row?;
        let mut row = row.into_iter();
        let label = row.next().unwrap().to_string();

        let label_measurements = all_measurements
            .entry(label.to_string())
            .or_insert_with(|| Measurements::new());
        label_measurements.count += 1;

        let values = row.map(|x| x.parse().unwrap());
        for (stat, value) in stat_names.iter().zip(values) {
            label_measurements
                .stats
                .entry(stat.clone())
                .or_insert_with(|| rolling_stats::Stats::new())
                .update(value);
        }

        let s = serde_json::to_string(&all_measurements)?;
        writeln!(stdout, "{}", s)?;
    }
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Measurements {
    pub count: usize,
    #[serde(flatten)]
    pub stats: BTreeMap<String, rolling_stats::Stats<f64>>,
}
impl Measurements {
    pub fn new() -> Measurements {
        Measurements {
            count: 0,
            stats: BTreeMap::new(),
        }
    }
    pub fn get(&self, stat: &str) -> Option<Stats> {
        let x = self.stats.get(stat)?;
        Some(Stats {
            count: self.count,
            mean: x.mean,
            std_dev: x.std_dev,
        })
    }
}
