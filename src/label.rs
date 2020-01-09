use ansi_term::{Color, Style};
use once_cell::sync::{Lazy, OnceCell};
use std::fmt;
use std::sync::Mutex;

static BENCH_CACHE: Lazy<Mutex<LabelCache>> = Lazy::new(|| Mutex::new(LabelCache::default()));

#[derive(Default)]
struct LabelCache(Vec<String>);

impl LabelCache {
    fn insert(&mut self, label: &str) -> usize {
        match self.0.iter().position(|x| x == label) {
            Some(x) => x,
            None => {
                self.0.push(label.to_string());
                self.0.len() - 1
            }
        }
    }
}

fn idx_to_color(idx: usize) -> Color {
    match idx % 4 {
        0 => Color::Purple,
        1 => Color::Yellow,
        2 => Color::Cyan,
        3 => Color::Green,
        _ => unreachable!(),
    }
}

#[derive(Debug, PartialEq, Clone, PartialOrd, Ord, Eq, Copy)]
pub struct Bench(pub usize);
impl From<&str> for Bench {
    fn from(x: &str) -> Bench {
        let mut cache = BENCH_CACHE.lock().unwrap();
        Bench(cache.insert(x))
    }
}
impl fmt::Display for Bench {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let cache = BENCH_CACHE.lock().unwrap();
        let color = idx_to_color(self.0);
        let s = &cache.0[self.0];
        write!(f, "{}", Style::new().fg(color).paint(s))
    }
}

pub fn all_benches() -> impl Iterator<Item = Bench> {
    (0..BENCH_CACHE.lock().unwrap().0.len()).map(Bench)
}

static METRIC_CACHE: OnceCell<Vec<String>> = OnceCell::new();

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

pub fn all_metrics() -> impl Iterator<Item = Metric> {
    (0..METRIC_CACHE.get().unwrap().len()).map(Metric)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_roudtrip() {
        assert_eq!(
            Bench::from("foobar").to_string(),
            "\u{1b}[35mfoobar\u{1b}[0m"
        );
        assert_eq!(Metric::from("zipzap").to_string(), "zipzap");
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
        assert_eq!(Metric::from("zipzap").to_string(), "zipzap");
    }
}
