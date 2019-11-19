# Continuous Benchmarking, Done Right

Continuous integration (which normally means "running the tests on every
commit") has become standard practise, and for good reason: if you don't
have good tests and run them regularly, you're bound to allow semantic
regressions (aka "bugs") into your code.  Continuous benchmarking is likewise
essential if you don't want to allow performance regressions to happen;
yet it's far less commonplace.

This is simply because it's harder: tests are deterministic, whereas benchmarks
are not.  Even "kinda deterministic" proxies such as instruction count are
quite variable in practice.  The property you care about (be it wall-time,
max RSS, or whatever) is not a number but is in fact a distribution, and the
property you _really_ care about is the mean of that distribution; and the
thing you _actually really_ care about is how much that mean changes when
you apply a particular patch.

## tl;dr: The Method

You have a decent macro-benchmark.  You have two commits you want to compare
(presumably: the merge-base vs. the tip of your branch).  You've decided
how small the smallest regression you care about is (call this `T`).

1. Pick a commit randomly and benchmark it; add the results to your total
   set of measurements.
2. Compute the 95% confidence interval for the diffence of the means.
3. Is the width of the confidence interval still bigger than `T`?  If so,
   go back to step 1.
4. Does the confidence interval contain zero?  If not, you may have a
   regression.

See [A Benchmarker's Journey](journey.md) for a story which explains one
might arrive at this method.

There's some stuff in this repo which might help you implement a scheme like
this yourself; see [here](cbdr.md) for an overview.

## Multiple benchmarks

If your benchmark produces multiple values (eg. wall time and max RSS),
then you want to check that none of them have regressed.  This multiplies
your chance of a false positive by the number of values involved.  You can
counteract this by multiplying the widths of your CIs by the same number.

## Stats

We're comparing the means of two unknown distributions.  Both distributions
are roughly normal, but their means and variances may be different.  (This is
typically the case when benchmarking; of course, if your benchmark calls
`sleep(random_log_normal())` then this isn't a valid assumption.  Don't do
that.)  The appropriate test in this case is Student's t-test.

We don't only care about whether a regression has occurred, however: we also
care about how big the regression is.  Therefore, a confidence interval is
going to be more useful than a p-value.  For this you'll need the inverse CDF
of the t-distribution.  There are various implementations around, including
one in this repo.

## Common bad practice

### Benchmarking commit A, then benchmarking commit B

Taking all measurements for one commit in a single session is tempting,
because it allows you to build the necessary artefacts, benchmark them, and
then delete them before moving on the next commit.  It has two disadvantages:

* You can't dynamically increase the number of samples based on the CI of
  the diff.  (You could take samples until the CI of the mean for the commit
  is small, but in the end you might find that the CI for the diff is still
  too wide.)
* Noise which fluctuates at the second- or minute-scale becomes correlated
  with commits.  Such sources of noise are very common in benchmarking rigs.
  Eg. imaging a cron jobs starts just as you start benchmarking commit B;
  the penalty is absorbed entirely by commit B.  If you randomize the
  measurements then the penalty will be even spread across both commits.

### Saving old benchmark results for later use

This is just a more extreme version of "benchmarking commit A, then
benchmarking commit B", except now your results are correlated with sources
of noise which vary at the day- or month-scale.  Is your cooling good enough
to ensure that results taken in the summer are comparable with results taken
in the winter?

#### But... I don't want to re-benchmark old commits!

Freshly benchmarking the base gives a number of advantages (in addition to
improving the resulting data, as explained above): your CI runs are now (1)
stateless and (2) machine-independent.  If you want to reuse old results,
you need to maintain a database and a dedicated benchmarking machine.
The downside of re-benchmarking is that your CI machines will spend - at most -
2x longer on benchmarking.  Is that a resource you really need to claw back?

### Plotting the means and eyeballing the difference

This is not a great idea, even when confidence intervals are included.
Quoting the [Biostats handbook]:

> There is a myth that when two means have confidence intervals that overlap,
> the means are not significantly different (at the P<0.05 level). Another
> version of this myth is that if each mean is outside the confidence
> interval of the other mean, the means are significantly different. Neither
> of these is true (Schenker and Gentleman 2001, Payton et al. 2003); it
> is easy for two sets of numbers to have overlapping confidence intervals,
> yet still be significantly different by a two-sample t–test; conversely,
> each mean can be outside the confidence interval of the other, yet they're
> still not significantly different. Don't try compare two means by visually
> comparing their confidence intervals, just use the correct statistical test.

[Biostats handbook]: http://www.biostathandbook.com/confidence.html

### Beware "±"

Just because a program prints its output with a "± x" doesn't mean it's
computing a confidence interval.  "±" could denote a standard deviatioon,
or some percentile, or anything really since "±" doesn't have a fixed
standardized meaning.  Having the variance of the measurements is well and
good, but it doesn't help you decide whether the result is significant.
If the docs don't specify the meaning, you could try grepping the source
for mention of an "inverse CDF".
