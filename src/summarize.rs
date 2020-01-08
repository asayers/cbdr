use crate::diff::*;
use crate::label::*;
use confidence::*;
use std::collections::BTreeMap;

#[derive(Default)]
pub struct Measurements(pub BTreeMap<(Label, String), Statistics>);

impl Measurements {
    pub fn update(&mut self, label: Label, values: impl Iterator<Item = (String, f64)>) {
        for (stat, value) in values {
            let Statistics(count, x) = self.0.entry((label.clone(), stat)).or_default();
            *count += 1;
            x.update(value);
        }
    }
    pub fn iter_label(&self, label: Label) -> impl Iterator<Item = (&str, &Statistics)> {
        self.0
            .range((label.clone(), "".to_string())..)
            .take_while(move |((l, _), _)| *l == label)
            .map(|((_, stat_name), stats)| (stat_name.as_str(), stats))
    }

    pub fn diff(&self, from: Label, to: Label) -> Diff {
        let xs = self.iter_label(from);
        let ys = self.iter_label(to);
        Diff::new(xs, ys)
    }

    pub fn guess_pairs(&self) -> Vec<(Label, Label)> {
        let mut scores: Vec<(Label, f64)> = vec![];
        let mut cur_score = 0.;
        let mut cur_label = None;
        for ((label, _), stats) in &self.0 {
            if Some(label) != cur_label {
                if let Some(l) = cur_label {
                    scores.push((l.clone(), cur_score));
                }
                cur_label = Some(label);
                cur_score = 0.;
            }
            cur_score += stats.1.mean;
        }
        if let Some(l) = cur_label {
            scores.push((l.clone(), cur_score));
        }
        scores.sort_by(|x, y| x.1.partial_cmp(&y.1).unwrap_or(std::cmp::Ordering::Equal));
        scores
            .iter()
            .zip(scores.iter().skip(1))
            .map(|(x, y)| (x.0.clone(), y.0.clone()))
            .collect()
    }
}

#[derive(Clone, Debug)]
pub struct Statistics(pub usize, pub rolling_stats::Stats<f64>);
impl Default for Statistics {
    fn default() -> Statistics {
        Statistics(0, rolling_stats::Stats::new())
    }
}
impl Into<Stats> for &Statistics {
    fn into(self) -> Stats {
        Stats {
            count: self.0,
            mean: self.1.mean,
            std_dev: self.1.std_dev,
        }
    }
}
