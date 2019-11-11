use serde::Deserialize;
use statrs::distribution::{StudentsT, Univariate};
use std::io::{BufWriter, Write};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let stdout = std::io::stdout();
    let mut stdout = BufWriter::new(stdout.lock());
    let mut rdr = csv::Reader::from_reader(std::io::stdin());
    let key_label = &rdr.headers()?[0];
    writeln!(stdout, "{},p-value", key_label)?;
    let mut last = None;
    for row in rdr.deserialize::<(String, usize, f64, f64)>() {
        let (key, count, mean, stddev) = row?;
        let stats = Stats {
            count,
            mean,
            stddev,
        };
        if let Some(last_stats) = last {
            let p_value = t_test(last_stats, stats);
            writeln!(stdout, "{},{}", key, p_value)?;
        }
        last = Some(stats);
    }
    Ok(())
}

#[derive(Deserialize, Clone, Copy)]
struct Stats {
    count: usize,
    mean: f64,
    stddev: f64,
}
impl Stats {
    fn var(&self) -> f64 {
        self.stddev * self.stddev
    }
}

/// Uses a one-tailed Welch's t-test to test whether the population mean of
/// xs2 is greater than the population mean of xs1.  Returns the probability
/// that the new mean is greater.
fn t_test(x1: Stats, x2: Stats) -> f64 {
    assert!(x1.count > 1);
    assert!(x2.count > 1);
    let foo = x1.var() / x1.count as f64 + x2.var() / x2.count as f64;
    let t = (x1.mean - x2.mean) / foo.sqrt();
    let v = degrees_of_freedom(x1, x2);
    let dist = StudentsT::new(0., 1., v).unwrap();
    dist.cdf(-t)
}

fn degrees_of_freedom(x1: Stats, x2: Stats) -> f64 {
    assert!(x1.count > 1);
    assert!(x2.count > 1);
    let foo = x1.var() / x1.count as f64 + x2.var() / x2.count as f64;
    let bar = x1.var() * x1.var() / (x1.count * x1.count * (x1.count - 1)) as f64;
    let qux = x2.var() * x2.var() / (x2.count * x2.count * (x2.count - 1)) as f64;
    foo * foo / (bar + qux)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        assert_eq!(0.00000000980976912232035, t_test(&BASE, &MEAN_1));
        assert_eq!(0.0014577657931561644, t_test(&BASE, &MEAN_4));
        assert_eq!(0.5, t_test(&BASE, &BASE));
        assert_eq!(0.029132411159442737, t_test(&BASE, &MEAN_5));
        assert_eq!(0.5475619774416596, t_test(&BASE, &MEAN_5_AGAIN));
        assert_eq!(0.7242791511167608, t_test(&BASE, &MEAN_6));
        assert_eq!(0.9999999871340238, t_test(&BASE, &MEAN_9));
    }

    #[test]
    fn test_high_var_base() {
        assert_eq!(0.00009155213843996927, t_test(&BASE_VAR, &MEAN_1));
        assert_eq!(0.13532525504796195, t_test(&BASE_VAR, &MEAN_4));
        assert_eq!(0.5, t_test(&BASE_VAR, &BASE_VAR));
        assert_eq!(0.4165749927586313, t_test(&BASE_VAR, &MEAN_5));
        assert_eq!(0.7765734787503631, t_test(&BASE_VAR, &MEAN_5_AGAIN));
        assert_eq!(0.8429073957562712, t_test(&BASE_VAR, &MEAN_6));
        assert_eq!(0.9999698957432896, t_test(&BASE_VAR, &MEAN_9));
    }

    // mean=5, sd=1
    const BASE: [f64; 10] = [5.25, 4.69, 4.87, 4.94, 5.87, 5.66, 5.76, 5.33, 6.63, 6.68];
    const MEAN_5: [f64; 10] = [4.45, 5.15, 4.65, 3.48, 6.18, 3.32, 4.9, 5.55, 5.32, 5.39];
    const MEAN_5_AGAIN: [f64; 10] = [5.24, 6.7, 4.86, 6.7, 4.51, 5.32, 5.7, 4.98, 4.16, 8.04];
    const MEAN_6: [f64; 10] = [5.88, 6., 5.45, 4.29, 7.41, 5.85, 6.17, 7.33, 4.63, 5.05];
    const MEAN_9: [f64; 10] = [9.24, 10.13, 9.76, 10.05, 8.0, 9.03, 9.47, 10.9, 7.85, 10.67];
    const MEAN_4: [f64; 10] = [4.62, 4.2, 3.97, 3.65, 3.34, 3.38, 3.48, 4.95, 6.67, 2.79];
    const MEAN_1: [f64; 10] = [-0.41, 0.68, 1.5, -0.49, 1.65, 1.49, 1.55, 2.95, -0.43, 1.57];

    // mean=5, sd=2
    const BASE_VAR: [f64; 10] = [5.66, 4.51, 2.05, 2.51, 3.5, 7.5, 6.54, 5.8, 3.26, 8.68];
}
