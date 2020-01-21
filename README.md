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

## An illustative example

For the sake of example, here's the situation: your repo contains a nice
macro-benchmark called `bench.sh`.  It only takes a second or so to run,
and it runs all the stuff you care about.  The rule is that if a feature
branch increases `bench.sh`'s running time by more than 2%, it should be
blocked from merging.  Here's the method:

1. Check out the feature branch and its merge-base in separate worktrees,
   and build whatever needs building.
2. Randomly pick one of your two checkouts and run `bench.sh`, measuring the
   time it takes to run.  Append the result to that checkout's "measurements"
   file.
3. Compute a one-tailed [confidence interval] for the difference of the means.
   The choice of α determines how many false-positive CI runs you're going to
   get, but note that a choosing higher confidence level means the confidence
   interval will shrink more slowly.
4. Look at the confidence interval as a percentage of the mean running time
   of the merge-base.
    * Is the upper bound less than +2%?  If so, you're good to merge!
    * Is the lower bound greater than +2%?  If so, it looks like there's
      a regression.
    * Otherwise, the confidence interval is too wide and you need more data:
      go back to step 2.
    * You may also want to include a time-limit.  Once it's reached, just
      check that the lower bound is above +2%.  If it is, there's no evidence
      of a regression, and it's probably safe to merge.

The above is just an example but hopefully you get the idea.  You can vary the
details; for instance, why not measure RSS instead of running time?  Or measure
RSS _as well_ as running time (but see "Comparing multiple values" below)?

If you want to implement something like this yourself, this repo contains a
[tool to help you do it](cbdr.md).

[confidence interval]: https://en.wikipedia.org/wiki/Welch%27s_t-test

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
in the winter?  (Ours isn't.)

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

### Using the wrong statistical test

The run-time of your benchmark is an unknown distrubution.  In my experience,
if you write a reasonable macro-benchmark, the distribution will be fairly
close to normal.  Of course, it's not _necessarily_ the case: if your
benchmark calls `sleep(random_log_normal())` then of course its runtime will
be log-normally distributed.  Do a normality test if you're concerned.

So: we have two distributions, assumed to be normal, but with different
means and variances.  We want to test whether the means are different.
The appropriate test in this case is Student's t-test.

However, we don't just want to know _whether_ a regression has occurred:
we want to know how big it is too.  A confidence interval is going to be
more useful than a p-value.  For this you'll need the inverse CDF of the
t-distribution.  There are various implementations around, including one in
this repo.

#### Beware "±"

Just because a program prints its output with a "± x" doesn't mean it's
computing a confidence interval.  "±" could denote a standard deviatioon,
or some percentile, or anything really since "±" doesn't have a fixed
standardized meaning.  Having the variance of the measurements is well and
good, but it doesn't help you decide whether the result is significant.
If the docs don't specify the meaning, you could try grepping the source
for mention of an "inverse CDF".

### Comparing multiple values and failing if any one of them regresses

If your benchmark produces multiple values (eg. wall time and max RSS), then
you probably want to check that none of them have regressed.  This multiplies
your chance of a false positive by the number of values involved.  You can
counteract this by multiplying the widths of your CIs by the same number.

#### Splitting the benchmark up

It's tempting to split your macro-benchmark up into micro-benchmarks so that
you can see what got slower.  I don't recommend this.  Microbenchmarks are a
great tool for exploratory benchmarking, but if you comparing them individually
you're going to have to widen the confidence intervals as mentioned above
(or suffer many false-positives), and it could take a really long time for
them to converge (if they ever converge enough at all).

In general, the tools used for performance _validation_ ("did it get
slower?") are not the same as the tools used for performance _profiling_
("which bit got slower?").  For the latter, [perf] ([see also]), [heap
profiling], [causal profiling], regression-based [micro-benchmarks], and
[flame graphs] are you friends.  But you don't need any of that fanciness to
validate overall performance - just one good macro-benchmark and a stopwatch.

[perf]: https://perf.wiki.kernel.org/
[see also]: http://www.brendangregg.com/perf.html
[heap profiling]: https://github.com/KDE/heaptrack
[causal profiling]: https://github.com/plasma-umass/coz
[micro-benchmarks]: http://www.serpentine.com/criterion/
[flame graphs]: https://github.com/llogiq/flame
