pub mod student_t;

/// Statictics for a sample taken from a normally-distributed population.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stats {
    pub count: usize,
    pub mean: f64,
    pub std_dev: f64,
}

impl Stats {
    fn var(self) -> f64 {
        self.std_dev * self.std_dev
    }
}

#[derive(Clone, PartialEq, Debug, Copy)]
pub struct ConfidenceInterval {
    pub center: f64,
    pub radius: f64,
}

impl fmt::Display for ConfidenceInterval {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} ± {}", self.center, self.radius)
    }
}

/// A confidence interval for `y.mean - x.mean`.
///
/// Given two normally distributed populations X ~ N(μ_x, σ²_x) and Y ~
/// N(μ_y, σ²_y), Y-X is distributed as N(μ_y - μ_x, σ²_x + σ²_y).
///
/// We have a sample from X and a sample from Y and we want to use these to
/// estimate μ_y - μ_x.
///
/// ## Variance of the difference between the means
///
/// When estimating μ with a sample mean ̄x, the variance of this estimate
/// is σ²/n, where n is the size of the sample.  Since we also don't know
/// σ², we have to estimate the variance of the estimated mean by s²/n.
/// In this case we want the variance of ̄y - ̄x.  We estimate it by
/// s²_x/n_x + s²_y/n_y.
///
/// ## Degrees of freedom
///
/// The degrees of freedom for s² is n-1.  To compute the pooled degrees
/// of freedom of the linear combination s²_x/n_x + s²_y/n_y, we use
/// the Welch–Satterthwaite equation.
pub fn confidence_interval(
    sig_level: f64,
    x: Stats,
    y: Stats,
) -> Result<ConfidenceInterval, Error> {
    // Prevent division by zero (see "degrees of freedom")
    if x.count < 2 || y.count < 2 {
        return Err(Error::NotEnoughData);
    }
    if !x.var().is_finite() || !y.var().is_finite() {
        return Err(Error::InfiniteVariance);
    }
    if x.var() == 0. || y.var() == 0. {
        return Err(Error::ZeroVariance);
    }

    // Convert `sig_level`, which is two-sided, into `p`, which is one-sided
    let alpha = 1. - sig_level;
    let p = 1. - (alpha / 2.);

    // Estimate the variance of the `y.mean - x.mean`
    let var = x.var() / x.count as f64 + y.var() / y.count as f64;

    // Approximate the degrees of freedom of `var_delta`
    let k_x = x.var() * x.var() / (x.count * x.count * (x.count - 1)) as f64;
    let k_y = y.var() * y.var() / (y.count * y.count * (y.count - 1)) as f64;
    let v = var * var / (k_x + k_y);

    // Compute the critical value at the chosen confidence level
    assert!(p.is_normal());
    assert!(v.is_normal());
    let t = student_t::inv_cdf(p, v);

    let center = y.mean - x.mean;
    let radius = t * var.sqrt();
    Ok(ConfidenceInterval { center, radius })
}

pub enum Error {
    NotEnoughData,
    InfiniteVariance,
    ZeroVariance,
}

use std::fmt;
impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::NotEnoughData => f.write_str("Can't compute CI when sample size is less than 2"),
            Error::InfiniteVariance => {
                f.write_str("The variance of one of the samples is infinite")
            }
            Error::ZeroVariance => f.write_str("The variance of one of the samples is zero"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cis() {
        let s1 = Stats {
            count: 10,
            mean: 5.,
            std_dev: 1.,
        };
        let s2 = Stats {
            count: 10,
            mean: 6.,
            std_dev: 1.5,
        };

        assert_eq!(
            confidence_interval(0.9, s1, s2).to_string(),
            "1 ± 0.9965524858858822"
        );
        assert_eq!(
            confidence_interval(0.95, s1, s2).to_string(),
            "1 ± 1.2105369242089183"
        );
        assert_eq!(
            confidence_interval(0.99, s1, s2).to_string(),
            "1 ± 1.6695970385386512"
        );
    }

    #[test]
    fn onlinestatbook() {
        // From http://onlinestatbook.com/2/estimation/difference_means.html
        let females = Stats {
            count: 17,
            mean: 5.353,
            std_dev: 2.743f64.sqrt(),
        };
        let males = Stats {
            count: 17,
            mean: 3.882,
            std_dev: 2.985f64.sqrt(),
        };
        assert_eq!(
            student_t_inv_cdf(0.975, 31.773948759590525),
            2.037501835321414
        );
        assert_eq!(
            confidence_interval(0.95, males, females).to_string(),
            "1.4709999999999996 ± 1.1824540265693928"
        );
    }

    #[test]
    fn zar() {
        // From Zar (1984) page 132
        let x = Stats {
            count: 6,
            mean: 10.,
            std_dev: 0.7206,
        };
        let y = Stats {
            count: 7,
            mean: 15.,
            std_dev: 0.7206,
        };
        assert_eq!(
            confidence_interval(0.95, x, y).to_string(),
            "5 ± 0.8854529371346332"
        );
    }

    // #[test]
    // fn nist() {
    //     // From the worked example at https://www.itl.nist.gov/div898/handbook/eda/section3/eda352.htm
    //     let x = Stats {
    //         count: 100,
    //         mean: 10.,
    //         std_dev: 0.022789,
    //     };
    //     let y = Stats {
    //         count: 95,
    //         mean: 19.261460,
    //         std_dev: 0.022789,
    //     };
    //     assert_eq!(
    //         confidence_interval(0.95, x, y).to_string(),
    //         "9.26146 ± 0.0032187032419323048"
    //     );
    // }
}
