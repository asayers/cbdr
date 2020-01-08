use crate::summarize::*;
use confidence::*;
use log::*;
use std::collections::BTreeMap;

pub struct Diff(pub BTreeMap<String, Option<DiffCI>>);
impl Diff {
    pub fn new<'a>(
        xs: impl Iterator<Item = (&'a str, &'a Statistics)>,
        ys: impl Iterator<Item = (&'a str, &'a Statistics)>,
    ) -> Diff {
        use itertools::*;
        Diff(
            xs.merge_join_by(ys, |x, y| x.0.cmp(&y.0))
                .map(|either| match either {
                    EitherOrBoth::Left((stat_name, _)) => (stat_name.to_string(), None),
                    EitherOrBoth::Right((stat_name, _)) => (stat_name.to_string(), None),
                    EitherOrBoth::Both((stat_name, x), (_, y)) => {
                        (stat_name.to_string(), DiffCI::new(x.into(), y.into()))
                    }
                })
                .collect(),
        )
    }
}

#[derive(Debug, Clone, PartialEq, Copy)]
pub struct DiffCI {
    pub stats_x: Stats,
    pub stats_y: Stats,
}
impl DiffCI {
    pub fn delta(self) -> f64 {
        self.stats_y.mean - self.stats_x.mean
    }
    pub fn ci(self, sig_level: f64) -> f64 {
        confidence_interval(sig_level, self.stats_x, self.stats_y).unwrap_or_else(|e| {
            match e {
                confidence::Error::NotEnoughData => (), // we expect some of these; ignore
                e => warn!(
                    "Skipping bad stats: {} ({:?} {:?})",
                    e, self.stats_x, self.stats_y
                ),
            };
            std::f64::NAN
        })
    }
    pub fn new(x: Stats, y: Stats) -> Option<DiffCI> {
        Some(DiffCI {
            stats_x: x,
            stats_y: y,
        })
    }
}
