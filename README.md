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

You have a nice macro-benchmark called `bench.sh`. It doesn't take too long to
run (a second or so), and it runs all the stuff you care about, in roughly the
right proportion.  Someone wants to merge their feature branch into master.
Here's what we do:

First, make two checkouts: one for the feature branch ("feature") and one
for its merge-base ("base"), and compile whatever needs to be compiled.

### Taking measurements

We're going to be taking multiple measurements and recording the results in
a CSV file.  "Taking a measurement" means doing the following:

1. Flip a coin.
2. If heads, run `base/bench.sh`.  If tails, run `feature/bench.sh`.
3. Record the time it took to run and append it to bench.csv, noting whether
   it was for "base" or "feature".

We're going to end up with a file which looks something like this:

```csv
branch    , wall_time
base      , 15.720428923
feature   , 16.173336192
base      , 15.488631299
feature   , 16.654012064
feature   , 16.37941706
feature   , 16.512443378
base      , 15.992080634
```

This doesn't need to be anything fancy: `/usr/bin/time --format "$branch,%e"
--append --output bench.csv $branch/bench.sh` has you covered - it's not
the most precice thing in the world but it's probably good enough.

### Computing a confidence interval

Now for the stats: From this data, we can compute a confidence interval
for the difference of the means using [Welch's t-test].  You'll need to
[choose an appropriate value for α][Choosing α].  You can then divide
the confidence interval by base's sample mean to get a percentage change
(optional, but it makes it more readable IMO).

For instance, if I compute the confidence interval using the data above, it
shows that going from `base` to `feature` changes the wall time by somewhere
between -5.8% and +14.6% - ie. we can't even really tell which one is faster!
Clearly more data is required.

[Welch's t-test]: https://en.wikipedia.org/wiki/Welch%27s_t-test
[Choosing α]: #choosing-α

There are packages available for julia, r, python, etc. that can help
you compute the confidence interval.  There's also [a CLI tool](cbdr.md)
available in this repo which can do it.

### The main loop

Ok, we're ready to begin.  Start by running `base/bench.sh` and
`feature/bench.sh` once each and throwing the results away.  These are just
"warm-ups" to get files cached etc.  Then:

1. Take a measurement.
2. Compute the confidence interval.
3.
   * If the whole interval is under +2% → You're good to merge!
   * If the whole interval is above +2% → It looks like there's a regression.
   * If the interval straddles +2% → We don't have enough data yet to tell
     whether there's a regression.  Go to step 1.

And that's it!  You may also want to include a time-limit.  What you do when
the you hit the time limit depends on how strict you are about performance.
Perhaps just check if the center is above or below 2%.  If you're regularly
hitting the time limit, investigate why your benchmark is so noisy.

The above is just an example but hopefully you get the idea.  You can vary the
details; for instance, why not measure max RSS as well as running time? (But
take note of [Checking too many variables] below.)

[Checking too many variables]: #-checking-too-many-variables

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

This is a fine way to benchmark, actually!  Sure, it's not exactly _rigorous_,
but on the plus side it's pretty hard to screw up.  However, this page is
about running benchmarks as part of your CI, so anything which requires a
human-in-the-loop is automatically out.

Do try plotting your raw benchmark data, though, just as a sanity check
(especially when first setting up your benchmarking CI).  You could plot
histograms like at the top of this page, or violin plots, or a heatmap.
Any of those will let you see at a glance how your benchmark is behaving.
(FYI [cbdr](cbdr.md) has functionality to help with this.)

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
deviatioon, or some percentile; but it could mean anything, really, since
the "±" symbol doesn't have a standardized meaning.

Having the variance of the measurements is well and good, but it doesn't
help you decide whether the result is significant.  If the docs don't specify
the meaning, you could try grepping the source for mention of an "inverse CDF".

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

# Justifying the method

## Choosing a family of distributions

Every time you run your benchmark you get a different result.  Those results
form a distrubution.  You can find out what this distrubution looks like by
running the benchmark a large number of times and plotting a histogram.

The shape depends on what your benchmark _does_, of course; but _in general_
benchmark results tend to have a lot of skew: on the left, it's as if there's
a hard minimum value which they can't do better than; but on the right it's
different: you get the occasional outlier which takes a really long time.

In my experience, you can usually get a decent fit using a log-normal. (But
perhaps something more kurtotic would be better?)

(Note: If your benchmark is just `sleep(random(10))`, for example, then
obviously its running time will be more-or-less uniformly distributed and
you're not going to get a good fit with a log-normal.  If you want to know the
shape of your benchmarks, do plot a histogram.  `cbdr` can help you with this.)

...actually, using log-normals remains future work.  For now, I'm going to
model them as _normally_ distributed.  It's not great.

## Choosing a statistic

So, we make the simplifying assumption that the "before" results and the
"after" results are coming from a pair of normal distributions with unknown
means and variances.  Now we can start trying to estimate the thing we're
interested in: do those distributions have the same mean?  The appropriate
test for this is Student's t-test.

IMO, a confidence interval is nicer to look at than a p-value, because it
gives some sense of how big the regression is (not just whether one exists).
For this we'll need the inverse CDF of the t-distribution.  There are various
implementations around, including one in this repo.

(Note: non-parametric tests do exist, but they're far less powerful than a
t-test, so it'll take much longer for your confidence intervals to shrink.)

## Choosing α

The choice of α determines how many false-positive CI runs you're going to get.
Choosing a higher confidence level means you'll get fewer false alarms, but it
also means that the confidence interval will take longer to shrink.  This could
just mean that your CI runs take longer; it could mean that they _never_ reach
the required tightness.

You probably only care about detecting _regressions_ and don't care about
detecting improvements; in this case you can use a one-tailed confidence
interval.

# What about measuring instruction count?

Some people use CPU counters to measure retired instructions, CPU cycles,
etc. as a proxy for wall time, in the hopes of getting more repeatable results.
There are two things to consider:

1. How well does your proxy correlate with wall time?
2. How much better is the variance, compared to wall time?

In my experience, simply countring instructions doesn't correlate well enough,
and counting CPU cycles is surprisingly high varience.  If you go down this
route I recommended you explore more sophisticated models, such as the one
used by [cachegrind].

If you do find a good proxy with less variance, then go for it!  Your
confidence intervals will converge faster.

[cachegrind]: https://valgrind.org/docs/manual/cg-manual.html

## Instruction count is ~~not determinisic~~ hard to make determinisic

The idea of swapping wall time for something 100% determinisic is very
tempting, because it means you can do away with all this statistical nonsense
and just compare two numbers.  Sounds good, right?

A previous version of this document claimed that there is no deterministic
measurement which is still a good proxy for wall time.  However, the Rust
project has recently made me eat my words.

Some [recent work][measureme PR] by the rustc people shows that it's possible
to get instruction count variance down almost all the way to zero.  It's very
impressive stuff, and the [writeup on the project][measureme writeup] is
excellent - I recommend reading it.

If you want to try this yourself, the tl;dr is that you need to count
instructions which retire in ring 3, and then subtract the number of
timer-based hardware interrupts.  You'll need:

* [x] a Linux setup with ASLR disabled;
* [x] a CPU with the necessary counters;
* [ ] a benchmark with deterministic _control flow_.

This last one is the catch.  Normally when we say a program is "deterministic"
we're referring to its observable output; but now the instruction count is
part of the observable output!

What does this mean in practice?  It means your program is allowed to
_look_ at the clock, /dev/urandom, etc... but it's not allowed to _branch_
on those things.  (Or, if it does, it had better make sure both branches
have the same number of instrucctions.)

This is a **very** hard pill to swallow, much harder than "simply" producing
deterministic output.  For example, many hashmap implementations mix some
randomness into their hashes.  A program which uses such a hashmap may
have the exact same behaviour every time it's run, but if you measure its
instruction count it will be different on every run.

The rustc team have gone to great lengths to ensure that (single-threaded)
rustc has this property.  For example, at some point rustc prints its own PID,
and the formatting code branches based on the number of digits in the PID.
This was a measureable source of variance and had to be fixed by padding
the formatted PID with spaces.  Yikes!

The conclusion: it _can_ be done; but if you're not willing to go all the
way like the Rust project did, then IMO you should still be estimating a
confidence interval.

[measureme PR]: https://github.com/rust-lang/measureme/pull/143
[measureme writeup]: https://hackmd.io/sH315lO2RuicY-SEt7ynGA?view
