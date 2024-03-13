/*!
A crate for testing whether the means of two Normal distributions are the same.

This crate implements [Welch's t-test], an approximate solution to the
[Behrens-Fisher problem].  The results are presented in the form of a
confidence interval.

[Welch's t-test]: https://en.wikipedia.org/wiki/Welch%27s_t-test
[Behrens-Fisher problem]: https://en.wikipedia.org/wiki/Behrens%E2%80%93Fisher_problem

## Example

Suppose we have a population distributed as `X` (normal), and another
distributed as `Y` (also normal, but possibly with different mean/variance to
`X`).  Let's take a sample from each population to estimate the difference
between the population means.

```
use behrens_fisher::*;
let x_sample: Vec<f64> = vec![1., 2., 3., 4.];
let y_sample: Vec<f64> = vec![3., 5., 7., 9., 11.];

let x_stats: SampleStats = x_sample.into_iter().collect();
let y_stats: SampleStats = y_sample.into_iter().collect();
let ci = difference_of_means(0.95, x_stats, y_stats).unwrap();
assert_eq!(ci.to_string(), "+4.50 ± 3.89 (p=95%)");
// Looks like μ[Y] > μ[X]!
```

*/

mod stats;
pub mod student_t;

pub use stats::*;
use std::fmt;

#[derive(Clone, PartialEq, Debug, Copy)]
pub struct ConfidenceInterval {
    /// The center of the two-sided confidence interval; the `x` in `x ± y`.
    pub center: f64,
    /// The half-width of the two-sided confidence interval; the `y` in
    /// `x ± y`.
    pub radius: f64,
    /// The significance level
    pub sig_level: f64,
}

impl fmt::Display for ConfidenceInterval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let prec = f.precision().unwrap_or(2);
        write!(
            f,
            "{:+.prec$} ± {:.prec$} (p={}%)",
            self.center,
            self.radius,
            self.sig_level * 100.0,
        )
    }
}

/// An estimate of μ_x (the population mean), based on a sample taken from X.
pub fn mean(sig_level: f64, x: SampleStats) -> Result<ConfidenceInterval, Error> {
    if sig_level <= 0.0 || sig_level >= 1.0 {
        return Err(Error::BadSigLevel);
    }
    if !x.var.is_finite() {
        return Err(Error::InfiniteVariance);
    }
    if x.var == 0. {
        return Err(Error::ZeroVariance);
    }

    // Convert `sig_level`, which is two-sided, into `p`, which is one-sided
    let alpha = 1. - sig_level;
    let p = 1. - (alpha / 2.);

    // The degrees of freedom of the mean variance
    let v = x.count as f64 - 1.0;

    // Compute the critical value at the chosen confidence level
    assert!(p.is_normal()); // "normal" in the f64 sense, not gaussian!
    assert!(v.is_normal()); // "normal" in the f64 sense, not gaussian!
    let t = student_t::inv_cdf(p, v);

    let center = x.mean;
    let radius = t * x.mean_var().sqrt();
    Ok(ConfidenceInterval {
        center,
        radius,
        sig_level,
    })
}

/// An estimate of `μ_y - μ_x` (the difference in population means),
/// based on samples taken from X and Y.
///
/// Given two normally distributed populations X ~ N(μ_x, σ²_x) and Y ~
/// N(μ_y, σ²_y), Y-X is distributed as N(μ_y - μ_x, σ²_x + σ²_y).
///
/// We have a sample from X and a sample from Y and we want to use these to
/// estimate μ_y - μ_x.
///
/// ## Variance of the difference between the means
///
/// We have an estimate of μ_(Y-X) - namely, ̄y - ̄x, and we want to
/// know the variance of that estimate.  For this we can use the sum of the
/// variances of ̄x and ̄y, which gives s²_x/n_x + s²_y/n_y.
///
/// ## Degrees of freedom
///
/// The degrees of freedom for s² is n-1.  To compute the pooled degrees
/// of freedom of the linear combination s²_x/n_x + s²_y/n_y, we use
/// the Welch–Satterthwaite equation.
pub fn difference_of_means(
    sig_level: f64,
    x: SampleStats,
    y: SampleStats,
) -> Result<ConfidenceInterval, Error> {
    if sig_level <= 0.0 || sig_level >= 1.0 {
        return Err(Error::BadSigLevel);
    }
    // Prevent division by zero (see "degrees of freedom")
    if x.count < 2 || y.count < 2 {
        return Err(Error::NotEnoughData);
    }
    if !x.var.is_finite() || !y.var.is_finite() {
        return Err(Error::InfiniteVariance);
    }
    if x.var == 0. || y.var == 0. {
        return Err(Error::ZeroVariance);
    }

    // Convert `sig_level`, which is two-sided, into `p`, which is one-sided
    let alpha = 1. - sig_level;
    let p = 1. - (alpha / 2.);

    // Estimate the variance of the `y.mean - x.mean`
    let x_mean_var = x.mean_var();
    let y_mean_var = y.mean_var();
    let var_delta = x_mean_var + y_mean_var;

    // Approximate the degrees of freedom of `var_delta`
    let k_x = x_mean_var * x_mean_var / (x.count - 1) as f64;
    let k_y = y_mean_var * y_mean_var / (y.count - 1) as f64;
    let v = var_delta * (var_delta / (k_x + k_y));

    // Compute the critical value at the chosen confidence level
    assert!(p.is_normal()); // "normal" in the f64 sense, not gaussian!
    assert!(v.is_normal()); // "normal" in the f64 sense, not gaussian!
    let t = student_t::inv_cdf(p, v);

    let center = y.mean - x.mean;
    let radius = t * var_delta.sqrt();
    Ok(ConfidenceInterval {
        center,
        radius,
        sig_level,
    })
}

#[derive(Debug, Clone, Copy)]
pub enum Error {
    BadSigLevel,
    NotEnoughData,
    InfiniteVariance,
    ZeroVariance,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::BadSigLevel => {
                f.write_str("The significance level must be between 0 and 1 (exclusive)")
            }
            Error::NotEnoughData => f.write_str("Can't compute CI when sample size is less than 2"),
            Error::InfiniteVariance => {
                f.write_str("The variance of one of the samples is infinite")
            }
            Error::ZeroVariance => f.write_str("The variance of one of the samples is zero"),
        }
    }
}
impl std::error::Error for Error {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cis() {
        let s1 = SampleStats {
            count: 10,
            mean: 5.,
            var: 1.,
        };
        let s2 = SampleStats {
            count: 10,
            mean: 6.,
            var: 2.25,
        };

        let ci = difference_of_means(0.9, s1, s2).unwrap();
        assert_eq!(ci.center, 1.0);
        assert_eq!(ci.radius, 0.9965524858858832);

        let ci = difference_of_means(0.95, s1, s2).unwrap();
        assert_eq!(ci.center, 1.0);
        assert_eq!(ci.radius, 1.2105369242089192);

        let ci = difference_of_means(0.99, s1, s2).unwrap();
        assert_eq!(ci.center, 1.0);
        assert_eq!(ci.radius, 1.6695970385386518);
    }

    #[test]
    fn onlinestatbook() {
        // From http://onlinestatbook.com/2/estimation/difference_means.html
        let females = SampleStats {
            count: 17,
            mean: 5.353,
            var: 2.743f64,
        };
        let males = SampleStats {
            count: 17,
            mean: 3.882,
            var: 2.985f64,
        };
        assert_eq!(
            student_t::inv_cdf(0.975, 31.773948759590525),
            2.037501835321414
        );
        let ci = difference_of_means(0.95, males, females).unwrap();
        assert_eq!(ci.center, 1.4709999999999996);
        assert_eq!(ci.radius, 1.1824540265693935);
        // the original example has it as 1.4709999999999996 ± 1.1824540265693928
        // the last two digits are different - probably just a rounding error
    }

    #[test]
    fn zar() {
        // From Zar (1984) page 132
        let x = SampleStats {
            count: 6,
            mean: 10.,
            var: (0.7206_f64).powf(2.),
        };
        let y = SampleStats {
            count: 7,
            mean: 15.,
            var: (0.7206_f64).powf(2.),
        };
        let ci = difference_of_means(0.95, x, y).unwrap();
        assert_eq!(ci.center, 5.0);
        assert_eq!(ci.radius, 0.885452937134633);
    }

    // #[test]
    // fn nist() {
    //     // From the worked example at https://www.itl.nist.gov/div898/handbook/eda/section3/eda352.htm
    //     let x = SampleStats {
    //         count: 100,
    //         mean: 10.,
    //         var: (0.022789).powf(2.),
    //     };
    //     let y = SampleStats {
    //         count: 95,
    //         mean: 19.261460,
    //         var: (0.022789).powf(2.),
    //     };
    //     assert_eq!(
    //         ConfidenceInterval::new(0.95, x, y).to_string(),
    //         "9.26146 ± 0.0032187032419323048"
    //     );
    // }
}
