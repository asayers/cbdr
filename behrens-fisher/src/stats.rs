use std::iter::FromIterator;

#[derive(Clone, Copy, Debug, PartialEq, Default)]
pub struct StatsBuilder {
    /// the number of samples seen so far
    count: usize,
    /// the mean of the entire dataset
    mean: f64,
    /// the squared distance from the mean
    m2: f64,
}

impl StatsBuilder {
    pub fn update(&mut self, x: f64) {
        // Welford's online algorithm
        self.count += 1;
        let delta1 = x - self.mean; // diff from the old mean
        self.mean += delta1 / self.count as f64;
        let delta2 = x - self.mean; // diff from the new mean
        self.m2 += delta1 * delta2;
    }

    pub fn count(self) -> usize {
        self.count
    }

    pub fn mean(self) -> f64 {
        if self.count == 0 {
            std::f64::NAN
        } else {
            self.mean
        }
    }

    pub fn sample_var(self) -> f64 {
        if self.count <= 1 {
            std::f64::NAN
        } else {
            self.m2 / (self.count - 1) as f64
        }
    }
}

impl Extend<f64> for StatsBuilder {
    fn extend<T: IntoIterator<Item = f64>>(&mut self, iter: T) {
        for x in iter {
            self.update(x);
        }
    }
}

/// Sample statictics.
///
/// Assumed to be taken from a normally-distributed population.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct SampleStats {
    /// The sample size
    pub count: usize,
    /// The sample mean
    pub mean: f64,
    /// The sample variance
    pub var: f64,
}

impl From<StatsBuilder> for SampleStats {
    fn from(x: StatsBuilder) -> SampleStats {
        SampleStats {
            count: x.count(),
            mean: x.mean(),
            var: x.sample_var(),
        }
    }
}

impl FromIterator<f64> for SampleStats {
    fn from_iter<T: IntoIterator<Item = f64>>(iter: T) -> SampleStats {
        let mut bldr = StatsBuilder::default();
        bldr.extend(iter);
        bldr.into()
    }
}

impl SampleStats {
    /// An estimate of the variance of `mean` (which is an estimate of the
    /// population mean).
    ///
    /// When estimating μ with a sample mean ̄x, the variance of this
    /// estimate is σ²/n, where n is the size of the sample.  Since we also
    /// don't know σ², we have to estimate the variance of the estimated
    /// mean by s²/n.
    pub fn mean_var(self) -> f64 {
        self.var / self.count as f64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test() {
        let stats = vec![1.0_f64, 2., 3.].into_iter().collect::<SampleStats>();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.mean, 2.);
        assert_eq!(stats.var, 1.);

        let stats = vec![0.0_f64, -2., 2.].into_iter().collect::<SampleStats>();
        assert_eq!(stats.count, 3);
        assert_eq!(stats.mean, 0.);
        assert_eq!(stats.var, 4.);

        let stats = (0..=100)
            .into_iter()
            .map(f64::from)
            .collect::<SampleStats>();
        assert_eq!(stats.count, 101);
        assert_eq!(stats.mean, 50.);
        assert_eq!(stats.var, 858.5);
    }
}
