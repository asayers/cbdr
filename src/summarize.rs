use crate::label::*;
use anyhow::*;
use confidence::*;
use serde::*;
use std::collections::BTreeMap;
use std::io::Write;

pub struct State {
    pub all_measurements: BTreeMap<Label, Measurements>,
}
impl State {
    pub fn new() -> State {
        State {
            all_measurements: BTreeMap::new(),
        }
    }
    pub fn update(&mut self, label: Label, values: impl Iterator<Item = (String, f64)>) {
        let label_measurements = self
            .all_measurements
            .entry(label)
            .or_insert_with(Measurements::new);
        for (stat, value) in values {
            label_measurements.update(stat, value);
        }
    }

    pub fn guess_pairs(&self) -> Vec<(Label, Label)> {
        let mut labels = self
            .all_measurements
            .iter()
            .map(|(label, measurements)| (label.clone(), measurements.score()))
            .collect::<Vec<_>>();
        labels.sort_by(|x, y| x.1.partial_cmp(&y.1).unwrap_or(std::cmp::Ordering::Equal));
        labels
            .iter()
            .zip(labels.iter().skip(1))
            .map(|(x, y)| (x.0.clone(), y.0.clone()))
            .collect()
    }
}

pub fn summarize() -> Result<()> {
    let mut rdr = csv::Reader::from_reader(std::io::stdin());
    let stat_names = rdr
        .headers()
        .unwrap()
        .into_iter()
        .skip(1)
        .map(|x| x.to_string())
        .collect::<Vec<_>>();
    let mut stdout = std::io::stdout();
    let mut state = State::new();
    for row in rdr.into_records() {
        let row = row?;
        let mut row = row.into_iter();
        let label = Label::from(row.next().unwrap().to_string());

        let values = row.map(|x| x.parse().unwrap());
        state.update(label, stat_names.iter().cloned().zip(values));
    }

    let s = serde_json::to_string(&state.all_measurements)?;
    writeln!(stdout, "{}", s)?;
    Ok(())
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Measurements(pub BTreeMap<String, (usize, rolling_stats::Stats<f64>)>);

impl Measurements {
    pub fn new() -> Measurements {
        Measurements(BTreeMap::new())
    }
    pub fn get(&self, stat: &str) -> Option<Stats> {
        let (count, x) = self.0.get(stat)?;
        Some(Stats {
            count: *count,
            mean: x.mean,
            std_dev: x.std_dev,
        })
    }
    fn update(&mut self, stat: String, value: f64) {
        let (count, x) = self
            .0
            .entry(stat)
            .or_insert_with(|| (0, rolling_stats::Stats::new()));
        *count += 1;
        x.update(value);
    }
    pub fn score(&self) -> f64 {
        self.0.values().map(|(_, stats)| stats.mean).sum::<f64>()
    }
}
