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

/// Statictics for a sample taken from a normally-distributed population.
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Stats {
    // The sample size
    pub count: usize,
    // The sample mean
    pub mean: f64,
    // The sample variance
    pub var: f64,
}

impl From<StatsBuilder> for Stats {
    fn from(x: StatsBuilder) -> Stats {
        Stats {
            count: x.count(),
            mean: x.mean(),
            var: x.sample_var(),
        }
    }
}

impl Stats {
    /// An estimate of the variance of `mean` (which is an estimate of the
    /// population mean).
    ///
    /// When estimating μ with a sample mean ̄x, the variance of this
    /// estimate is σ²/n, where n is the size of the sample.  Since we also
    /// don't know σ², we have to estimate the variance of the estimated
    /// mean by s²/n.
    pub(crate) fn mean_var(self) -> f64 {
        self.var / self.count as f64
    }
}