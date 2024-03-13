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
the most precise thing in the world but it's probably good enough.

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
form a distribution.  You can find out what this distribution looks like
by running the benchmark a large number of times and plotting a histogram
(or better yet: a kernel density estimate).

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

Ok, so let's assume that our distributions are log-normal.  Well
actually... the method described above uses a t-test, which assumes they're
normally distributed.  It's not great.  If anyone knows a test for the
difference of the means of two log-normally distributed populations, please
let me know!

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

The choice of α determines how many false-positive CI runs you're going to
get.  Choosing a higher confidence level means you'll get fewer false alarms,
but it also means that the confidence interval will take longer to shrink.
This could just mean that your CI runs take longer; but it could mean that
they _never_ reach the required tightness.

You probably only care about detecting _regressions_ and don't care about
detecting improvements; in this case you can use a one-tailed confidence
interval.
