use ansi_term::{Color, Style};
use arc_swap::ArcSwap;
use serde::{Serialize, Serializer};
use std::fmt;
use std::sync::{LazyLock, OnceLock};

static BENCH_CACHE: LazyLock<ArcSwap<Vec<String>>> = LazyLock::new(ArcSwap::default);

#[derive(Debug, PartialEq, Clone, PartialOrd, Ord, Eq, Copy)]
pub struct Bench(pub usize);

impl From<&str> for Bench {
    fn from(x: &str) -> Bench {
        match BENCH_CACHE.load().iter().position(|y| x == y) {
            Some(x) => Bench(x),
            None => {
                let old = BENCH_CACHE.rcu(|cache| {
                    let mut cache = Vec::clone(cache);
                    cache.push(x.to_string());
                    cache
                });
                Bench(old.len())
            }
        }
    }
}

impl fmt::Display for Bench {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let color = match self.0 % 4 {
            0 => Color::Purple,
            1 => Color::Yellow,
            2 => Color::Cyan,
            3 => Color::Green,
            _ => unreachable!(),
        };
        let cache = BENCH_CACHE.load();
        let s = &cache[self.0];
        write!(f, "{}", Style::new().fg(color).paint(s))
    }
}

impl Serialize for Bench {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let cache = BENCH_CACHE.load();
        let s = &cache[self.0];
        serializer.serialize_str(s)
    }
}

pub fn all_benches() -> impl Iterator<Item = Bench> {
    (0..BENCH_CACHE.load().len()).map(Bench)
}

static METRIC_CACHE: OnceLock<Vec<String>> = OnceLock::new();

pub fn init_metrics(metrics: Vec<String>) {
    METRIC_CACHE.set(metrics).unwrap()
}

#[derive(Debug, PartialEq, Clone, PartialOrd, Ord, Eq, Copy)]
pub struct Metric(pub usize);
impl fmt::Display for Metric {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let cache = METRIC_CACHE.get().unwrap();
        f.write_str(&cache[self.0])
    }
}

impl Serialize for Metric {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let cache = METRIC_CACHE.get().unwrap();
        let s = &cache[self.0];
        serializer.serialize_str(s)
    }
}

pub fn all_metrics() -> impl Iterator<Item = Metric> {
    (0..METRIC_CACHE.get().unwrap().len()).map(Metric)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bench_roundtrip() {
        assert_eq!(
            Bench::from("foobar").to_string(),
            "\u{1b}[35mfoobar\u{1b}[0m"
        );
        assert_eq!(
            Bench::from("barqux").to_string(),
            "\u{1b}[33mbarqux\u{1b}[0m"
        );
        assert_eq!(
            Bench::from("barqux").to_string(),
            "\u{1b}[33mbarqux\u{1b}[0m"
        );
        assert_eq!(
            Bench::from("foobar").to_string(),
            "\u{1b}[35mfoobar\u{1b}[0m"
        );
    }

    #[test]
    fn test_metric_roundtrip() {
        init_metrics(vec!["foobar".into(), "barqux".into()]);
        assert_eq!(Metric(0).to_string(), "foobar");
        assert_eq!(Metric(1).to_string(), "barqux");
    }
}
