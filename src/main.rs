use statrs::distribution::{StudentsT, Univariate};
use statrs::statistics::Statistics;
use std::io::{BufWriter, Write};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

fn main() -> Result<()> {
    let stdout = std::io::stdout();
    let mut stdout = BufWriter::new(stdout.lock());
    let mut rdr = csv::Reader::from_reader(std::io::stdin());

    // Parse the header row
    let mut hdrs = rdr.headers()?.into_iter();
    let key_hdr = hdrs.next().unwrap();
    eprintln!("Grouping by {}", key_hdr);
    write!(stdout, "{}", key_hdr)?;
    let mut num_cols = 0;
    for hdr in hdrs {
        num_cols += 1;
        write!(stdout, ",{}", hdr)?;
    }
    let mut vals: Vec<f64> = vec![];

    // handle the first group
    let mut row = csv::StringRecord::new();
    rdr.read_record(&mut row)?;
    let mut this_key: String = row[0].to_string();
    handle_row(&mut this_key, &mut vals, &row, &mut stdout)?;

    for row in rdr.into_records() {
        println!("ok");
    }
    Ok(())
}

fn handle_row(this_key: &mut String, vals: &mut Vec<f64>, row: &csv::StringRecord, mut stdout: impl Write) -> Result<()> {
    let mut row_iter = row.iter();
    let key = row_iter.next().unwrap();
    if key != *this_key {
        write!(stdout, "done for {}", this_key)?;
        *this_key = key.to_string();
        vals.clear();
    }
    for x in row_iter {
        vals.push(x.parse::<f64>()?);
    }
    Ok(())
}

/// Uses a one-tailed Welch's t-test to test whether the population mean of
/// xs2 is greater than the population mean of xs1.  Returns the probability
/// that the new mean is greater.
pub fn t_test(xs1: &[f64], xs2: &[f64]) -> f64 {
    assert!(xs1.len() > 1);
    assert!(xs2.len() > 1);
    let n1 = xs1.len();
    let n2 = xs2.len();
    let mean1 = xs1.mean();
    let mean2 = xs2.mean();
    let var1 = xs1.variance();
    let var2 = xs2.variance();
    let foo = var1 / n1 as f64 + var2 / n2 as f64;
    let t = (mean1 - mean2) / foo.sqrt();
    let v = foo * foo
        / (var1 * var1 / (n1 * n1 * (n1 - 1)) as f64 + var2 * var2 / (n2 * n2 * (n2 - 1)) as f64);
    let dist = StudentsT::new(0., 1., v).unwrap();
    dist.cdf(-t)
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
