use confidence::*;
use log::*;

pub struct Diff(pub Vec<DiffCI>);

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
}
