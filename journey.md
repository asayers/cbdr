# The benchmarker's journey

OK, so let's assume you have a decent macro benchmark.  There's a script in
your repo called `bench.sh`, and when you run it it goes off and thinks for
about a second and comes back with some results:

```
$ ./bench.sh
{
    "dump_pcap": {
        "task_clock": 288.056773,
        "mega-cycles": 1231.419695,
        "mega-instructions": 2075.278076,
    },
    "compute_checksums": {
        "task_clock": 1.557046,
        "mega-cycles": 6.503665,
        "mega-instructions": 6.467289,
    }
}
```

Great.  Let's run it in our CI and report the results.  The developer
can then compare them to the merge-base and see if there's a regression.
Well... obviously this isn't very practical, so let's compute it for them:

```
dump_pcap:
    task_clock:          288.056 ->  288.056
    mega-cycles:        1231.419 -> 1231.419
    mega-instructions:  2075.278 -> 2075.278
compute_checksums:
    task_clock:            1.557 ->    1.557
    mega-cycles:           6.503 ->    6.503
    mega-instructions:     6.467 ->    6.467
```

A bit better... but we all know these values are going to be so noise that
they're basically useless.  How about running the benchmark script 100 times
are using the means instead?  We can even throw in the variance of the mean
for good measure.  (We can estimate this by the sample variance divided by
the sample size.)

```
dump_pcap:
    task_clock:          288.056 ±  21.805 ->  328.506 ±  28.085
    mega-cycles:        1231.419 ± 102.141 -> 2231.419 ± 132.114
    mega-instructions:  2075.278 ± 320.527 -> 3075.738 ± 270.357
compute_checksums:
    task_clock:            1.557 ±   0.255 ->    1.957 ±   0.716
    mega-cycles:           6.503 ±   0.560 ->    7.806 ±   0.406
    mega-instructions:     6.467 ±   0.646 ->    7.628 ±   0.466
```

But what we really want to know is whether the values have changed, and
whether they've changed by a lot (relatively).  So let's compute the delta
as a percentage of the base value.  I'm also estimating the variance of the
delta (which is the sum of the variances of the means), and converting that
into a percentage of the base as well.

```
dump_pcap:
    task_clock:         +14.042% ± 0.197%
    mega-cycles:        +81.207% ± 0.207%
    mega-instructions:  +48.208% ± 0.230%
compute_checksums:
    task_clock:         +25.690% ± 0.559%
    mega-cycles:        +20.036% ± 0.186%
    mega-instructions:  +17.952% ± 0.171%
```

This is starting to look a bit easier to read... but it's still hard to know
whether these values are statistically significant.  I mean, it looks like
the clock times have gone up - but do we have enough data to be confident?

This sounds like a hypothesis test to me.  Specifically, we're want to know
whether the means of two distributions are the same.  The distributions
may have different variances, but we can assume that they're roughly normal
(benchmark results usually are).  For this, we need Welch's t-test.

Since we're interested in how much the values have changed, as well as whether
they're different, we want to show confidence intervals rather than p-values.

```
dump_pcap:
    task_clock:         [ +13.845%  +14.239% ]
    mega-cycles:        [ +81.000%  +81.414% ]
    mega-instructions:  [ +47.978%  +48.438% ]
compute_checksums:
    task_clock:         [ +25.131%  +26.249% ]
    mega-cycles:        [ +19.850%  +20.222% ]
    mega-instructions:  [ +17.781%  +18.123% ]
```

And there we have it: we can now proclaim, for example, that dump_pcap's run
time increased by 13.8-14.2% (p=95%).  That's exactly the kind of statement
we wanted to be able to make.
