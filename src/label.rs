use ansi_term::{Color, Style};
use once_cell::sync::Lazy;
use std::fmt;
use std::sync::Mutex;

static BENCH_CACHE: Lazy<Mutex<LabelCache>> = Lazy::new(|| Mutex::new(LabelCache::default()));
static METRIC_CACHE: Lazy<Mutex<LabelCache>> = Lazy::new(|| Mutex::new(LabelCache::default()));

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
pub struct Bench(usize);
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

#[derive(Debug, PartialEq, Clone, PartialOrd, Ord, Eq, Copy)]
pub struct Metric(usize);
impl Metric {
    pub const MIN: Metric = Metric(0);
    pub const MAX: Metric = Metric(std::usize::MAX);
}
impl From<&str> for Metric {
    fn from(x: &str) -> Metric {
        let mut cache = METRIC_CACHE.lock().unwrap();
        Metric(cache.insert(x))
    }
}
impl fmt::Display for Metric {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let cache = METRIC_CACHE.lock().unwrap();
        f.write_str(&cache.0[self.0])
    }
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
