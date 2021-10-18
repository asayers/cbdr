**If you're looking for `cbdr`, a tool which helps implement this method,
look [here](cbdr.md) instead.**

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
take note of [Checking too many variables] on the main page.)

[Checking too many variables]: README.md#-checking-too-many-variables

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
