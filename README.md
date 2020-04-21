<p align="center"> <img src="banner.png" /> </p>
<h1 align="center">Continuous Benchmarking, Done Right</h1>

Continuous integration has become standard practice, and for good reason:
if you don't run your tests regularly then it's just a matter of time before
"semantic regressions" (aka bugs) find their way into master.  Likewise,
if you don't run your benchmarks regularly then performance regressions
**will** happen.

Yet running benchmarks in CI is far less common.  The reason: tests are
deterministic but benchmarks are not, and this makes it hard to boil them
down to a pass/fail.  Even "kinda deterministic" proxies such as instruction
count are quite variable in practice.

There's some number you care about (running time, max RSS, etc.); except
it's **not** a number but a distribution, and the number you _actually_ care
about is the mean of that distribution; and the thing you _really actually_
care about is how much that mean changes when you apply some particular patch.
(This is known in the business as the [Behrens–Fisher problem].)

[Behrens–Fisher problem]: https://en.wikipedia.org/wiki/Behrens%E2%80%93Fisher_problem

This page contains some assorted advice on how to measure this.  **If you're
looking for the `cbdr` tool, look [here](cbdr.md) instead.**

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

### Checking-in benchmark thresholds

Some projects keep benchmark thresholds checked in to their repo, and fail in
CI when those thresholds are exceeded.  I've already argued against storing
previous benchmark results, but you _might_ consider checking in a certain
old version of your code to compare your benchmark against.  For instance,
you might have a file containing a revision number; CI benchmarks against
that revision; if HEAD is slower, CI fails.  If you're OK with the performance
of HEAD, you put a newer (slower) revision in the file and the CI passes.

This has the advantage that it allows you to detect slow long-term performance
creep.  However, this kind of creep is very rarely actionable.  In practice,
since the failing commit isn't _really_ to blame, you just update the threshold
and move on.  The GHC people had a system like this in place for a long time
but [recently ditched it][GHC] because it was just seen as a nuisanse.

[GHC]: https://gitlab.haskell.org/ghc/ghc/wikis/performance/tests

### Combining results from different benchmarks

The method above involves running a "good macrobenchmark".  Suppose you don't
have one of those, but you _do_ have lots of good microbenchmarks.  How about
we take a set of measurements separately for each benchmark and the combine
them into a single set?  (ie. concatenate the rows of the various csv files.)

Well, this fine, but there's one problem: the distribution of results probably
won't be very normal - it's likely to be multi-modal.  You _can_ compare these
lumpy distributions to each other, but you can't use a t-test.  You'll have
to carefully find an appropriate test, and it will probably be less powerful.

Instead, why not make a macrobenchmark which runs all of the microbenchmarks
in sequence?  This will take all your microbenchmarks into account, and give
you a far more gaussian-looking distribution.
