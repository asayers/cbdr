<p align="center"> <img src="banner.png" /> </p>
<h1 align="center">Continuous Benchmarking, Done Right</h1>

Run benchmarks as part of your continuous integration; automatically flag
MRs which regress performance.

Sounds good, right?  So why is it so uncommon to see benchmarks in CI?
The problem is that it's hard to boil it down to a simple "pass/fail".
CI runners are typically very noisy, so you need to be careful if you want
reliable results.

It's hard, but if you care about performance then it's worth it.  Without
benchmarks in your CI, accidental regressions _will_ happen.  It's easier
to fix them when they're fresh.

Here's the situation:

* there's some number you care about: CPU seconds, max memory usage, whatever;
* except it's _not_ really a number - it's a distribution - and the number
  you _actually_ care about is the mean of that distribution;
* and the thing you _actually actually_ care about is not the mean per-se,
  but how much it'll change if you merge your branch.

In other words, there are two unknown distributions ("before" and "after"),
and our challenge is to estimate how much their means differ by sampling from
them repeatedly.  (This is known in the business as the [Behrens–Fisher
problem], assuming the distributions are roughly Normal.)

This page contains some assorted advice on how to do this.  **If you're
looking for `cbdr`, a tool which automates some of this advice, look
[here](cbdr.md) instead.**

[Behrens–Fisher problem]: https://en.wikipedia.org/wiki/Behrens%E2%80%93Fisher_problem

# The method

Let's suppose your repo contains a nice macro-benchmark called `bench.sh`. It
only takes a second or so to run, and it runs all the stuff you care about, in
roughly the right proportion.  Let's also suppose that we want to see a CI
warning if a feature branch increases `bench.sh`'s running time by more than 2%.
Here's what the CI job does:

1. Check out the feature branch and its merge-base in separate worktrees -
   we'll call them "master" and "feature" - and build whatever needs building.
2. Flip a coin.
    * If heads, run `master/bench.sh`
    * If tails, run `feature/bench.sh`

   Record the time it takes to run and add it to the set of measurements.
3. Using Welch's t-test, compute a one-tailed [confidence interval] for the
   difference of the means.  (See [Choosing α] below.)
4. Divide the confidence interval by the master's mean running time to get a
   percentage.
    * The upper bound is below +2% → You're good to merge!
    * The lower bound is above +2% → It looks like there's a regression.
    * The interval contains +2% → The confidence interval is too wide and you
      need more data: go to step 2.

[Choosing α]: #choosing-α

The above is just an example but hopefully you get the idea.  You can vary the
details; for instance, why not measure max RSS as well as running time? (But see
[Checking too many variables] below.)

[Checking too many variables]: #-checking-too-many-variables

You may also want to include a time-limit.  Once it's reached, just check that
the lower bound is above +2%.  If it is, there's no evidence of a regression,
and it's probably safe to merge.

Implementing step 2 is easy: `time --format=%e --append --output=measurements`
has you covered - it's not the most precice thing in the world but it's probably
good enough.

For the t-test you could use julia or python; alternatively, this repo contains
a [tool to help you do it](cbdr.md).

[confidence interval]: https://en.wikipedia.org/wiki/Welch%27s_t-test

# Common bad practice

## Part 1: Sampling

### ❌ Benchmarking commit A, then benchmarking commit B

Most benchmarking software I see takes a bunch of measurements of the first
thing, then a bunch of measurements of the second thing, then performs some
analysis on the two samples.  I suggest that you don't do this; instead,
randomly pick one of your commits to measure each time you do a run.
This has two advantages:

* If results are correlated with time, then they're correlated with
  time-varying noise.  Imagine you finish benchmarking commit A on a quiet
  machine, but then a cron jobs starts just as you start benchmarking commit B;
  the penalty is absorbed entirely by commit B!  This applies to intermittent
  noise which varies at the second- or minute-scale, which is very common
  in benchmarking rigs.

  On the other hand, if you randomize the measurements then the penalty of
  the cron job in the example above will be even spread across both commits.
  It will hurt the precision of your results, but not the accuracy.
* You can dynamically increase the number of samples until you achieve a
  desired presicion.  After each sample you look at how wide the confidence
  interval of the mean-difference is, and if it's too wide you take another
  measurement.

  If you perform the measurements in order, then at the time you decide to
  move on from commit A to commit B, you only have access to the confidence
  interval of the mean for commit A at the start, not the mean-difference.
  Just because you have a precice estimate of the mean for commit A, it
  doesn't mean you're going to have enough data for a precice estimate of
  the mean-difference.

I get it: you want to build commit A, benchmark it, then build commit B in
the same checkout (replacing the artifacts from commit A).  Just save the
artifacts somewhere.  I use $HOME/.cache/$PROJECT_bench/$SHORTREV.

### ❌ Saving old benchmark results for later use

This is just a more extreme version of "benchmarking commit A, then
benchmarking commit B", except now your results are correlated with sources of
noise which vary at the day- or month-scale.  Is your cooling good enough to
ensure that results taken in the summer are comparable with results taken in
the winter?  (Ours isn't.)  Did you upgrade any hardware between now and then?
How about software?

In addition to improving the quality of your data, freshly benchmarking the
base means your CI runs are now (1) stateless and (2) machine-independent.
If you want to reuse old results, you need to maintain a database and a
dedicated benchmarking machine.  If you re-bench the base, you can use any
available CI machine and all it needs is your code.

Re-benchmarking old commits feels like a waste of CPU-time; but your CI
machines will spend - at most - 2x longer on benchmarking.  Is that _really_
a resource you need to claw back?

### ⚠ Checking-in benchmark thresholds

Some projects keep benchmark thresholds checked in to their repo, and fail
in CI when those thresholds are exceeded.  If the slowdown is expected,
you update the thresholds in the same PR.  The idea is that it allows you
to detect slow, long-term performance creep.  It's a nice idea.

I've already argued against storing old benchmark results generally, but we
can fix that: instead of checking in results, we could check in a reference
to some old version of the code to benchmark against.

This scheme does indeed allow you to detect performance creep.  However,
this kind of creep is very rarely actionable.  Usually the failing commit
isn't _really_ to blame - it's just the straw that broke the camel's back -
so in practice you just update the threshold and move on.  Once this becomes
a habit, your benchmarks are useless.  The GHC people had a system like this
in place for a long time but [recently ditched it][GHC] because it was just
seen as a nuisance.

[GHC]: https://gitlab.haskell.org/ghc/ghc/wikis/performance/tests

## Part 2: Analysis

### ❌ Plotting the results and eyeballing the difference

That's... well actually it's not the worst way to benchmark.  Sure, it's
not exactly _rigorous_, but on the plus side it's pretty hard to screw up.
However, this page is about running benchmarks as part of your CI, so anything
which requires a human-in-the-loop is automatically out.

### ❌ Computing the two means and comparing them

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

### ⚠️ Beware "±"

Just because a benchmarking library prints its output with a "± x" doesn't
mean it's computing a confidence interval.  "±" often denotes a standard
deviatioon, or some percentile, or anything really since "±" doesn't have a
fixed standardized meaning.  Having the variance of the measurements is well
and good, but it doesn't help you decide whether the result is significant.
If the docs don't specify the meaning, you could try grepping the source
for mention of an "inverse CDF".

## Part 3: Validation vs exploration

### ❌ Checking too many variables

If your benchmark measures multiple things (eg. wall time and max RSS) then you
probably want to check all of them to make sure that that nothing has regressed.
Beware, however: the chance of a false positive will increase by a multiple of
the number of values involved.  You can counteract this by multiplying the
widths of your CIs by the same number, but this means that they'll take longer
to shrink.

Running all of your microbenchmarks in CI and comparing them individually sounds
like a good idea ("I'll know which component got slower!"), but in practice
you'll get so many false positives that you'll start ignoring CI failures.
Instead, CI should just check overall performance.  If there's a regression, you
crack out the microbenchmarks to figure out what caused it.

Analogy: your test suite tried to answer the question: "is it broken?".  If the
answer is "yes" then you crack out GDB or whatever to answer the question: "what
exactly is broken?".

Likewise, we're trying to answer the question: "did it get slower?".  For that,
all you need is one good macro-benchmark and a stopwatch. If the answer is
"yes", then it's time for [perf] ([examples]), [heap profiling], [causal
profiling], regression-based [micro-benchmarks], [flame graphs], [frame
profiling][tracy], etc..

[perf]: https://perf.wiki.kernel.org/
[examples]: http://www.brendangregg.com/perf.html
[heap profiling]: https://github.com/KDE/heaptrack
[causal profiling]: https://github.com/plasma-umass/coz
[micro-benchmarks]: http://www.serpentine.com/criterion/
[flame graphs]: https://github.com/llogiq/flame
[tracy]: https://github.com/wolfpld/tracy

### ❌ Concatenating results from different benchmarks

The method above involves running a "good macrobenchmark".  Suppose you don't
have one of those, but you _do_ have lots of good microbenchmarks.  How about
we take a set of measurements separately for each benchmark and the combine
them into a single set?  (ie. concatenate the rows of the various csv files.)

There's a problem with this: the distribution of results probably won't be
very normal; in fact it's likely to be highly multi-modal (aka. lumpy).
You _can_ compare these lumpy distributions to each other, but not with
a t-test.  You'll have to carefully find an appropriate test (probably a
non-parametric one), and it will probably be a lot less powerful.

Instead, why not make a macrobenchmark which runs all of the microbenchmarks
in sequence?  This will take all your microbenchmarks into account, and give
you a far more gaussian-looking distribution.

# Misc

## Choosing a statistic

The run-time of your benchmark is an unknown distrubution.  In my experience, if
you write a reasonable macro-benchmark, the distribution will be fairly close to
normal.  This is not _necessarily_ the case, of course!  If your benchmark calls
`sleep(random_log_normal())` then its running time will be (surprise surprise)
log-normally distributed.  Do a normality test if you're concerned.

So: we have two distributions, assumed to be normal, but with different
means and variances.  We want to test whether the means are different.
The appropriate test in this case is Student's t-test.

However, we don't just want to know _whether_ a regression has occurred:
we want to know how big it is too.  A confidence interval is going to be
more useful than a p-value.  For this you'll need the inverse CDF of the
t-distribution.  There are various implementations around, including one in
this repo.

If your benchmark gives timings which are not at all normally-distributed
(and you don't feel like changing your benchmark), non-parametric tests
do exist.  They're far less powerful than a t-test, though, so it'll take
much longer for your confidence intervals to shrink.

## Choosing α

The choice of α determines how many false-positive CI runs you're going to get.
Choosing a higher confidence level means you'll get fewer false alarms, but it
also means that the confidence interval will take longer to shrink.  This could
just mean that your CI runs take longer; it could mean that they _never_ reach
the required tightness.

## What about measuring instruction count?

Some people measure "kinda deterministic" proxies such as instructions or
CPU cycles instead of wall time.  In my experience, these do indeed have
less variance than wall time, but it's far from zero.  Furthermore, they're
less well-correlated with wall time than you might expect.  That said, if
variance is a real problem for you (ie. your confidence intervals are taking
a long time to shrink) then you could try switching to instruction count.
Just be aware that you're no longer quite measuring the thing you care about.
