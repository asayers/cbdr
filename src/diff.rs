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
    pub fn new(x: Stats, y: Stats) -> Option<DiffCI> {
        Some(DiffCI {
            mean_x: x.mean,
            mean_y: y.mean,
            r95: mk_ci(0.95, x, y)?,
            r99: mk_ci(0.99, x, y)?,
        })
    }
}

fn mk_ci(sig_level: f64, x: Stats, y: Stats) -> Option<f64> {
    confidence_interval(sig_level, x, y)
        .map_err(|e| match e {
            confidence::Error::NotEnoughData => (), // we expect some of these; ignore
            e => warn!("Skipping bad stats: {} ({:?} {:?})", e, x, y),
        })
        .ok()
}
