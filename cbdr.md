# CBDR

This repo contains a suite of tools called `cbdr`.

`cbdr sample` repeatedly selects a program at random from a list and
benchmarks it.  The output is CSV-formatted.

```
$ cbdr sample 'md5sum foo.json' 'sha1sum foo.json' 'sha256sum foo.json' | head | column -s, -t
Warming up md5sum foo.json...
Warming up sha1sum foo.json...
Warming up sha256sum foo.json...

target              sys time  user time  wall time
sha1sum foo.json    0.01      0.04       0.038843459
md5sum foo.json     0         0.03       0.035293018
md5sum foo.json     0         0.03       0.036419416
md5sum foo.json     0.01      0.03       0.031535277
sha256sum foo.json  0         0.1        0.099074689
md5sum foo.json     0         0.02       0.031327095
md5sum foo.json     0         0.03       0.031426344
sha256sum foo.json  0         0.09       0.087450079
md5sum foo.json     0         0.03       0.030244706
```

`cbdr analyze` summarizes the differences between the benchmarked programs.

```
$ cbdr analyze <results.csv
benchmark           samples  sys time  user time  wall time
md5sum foo.json     7238     0.003     0.025      0.028
sha1sum foo.json    7057     0.003     0.031      0.034
sha256sum foo.json  7200     0.003     0.074      0.077

md5sum foo.json vs sha1sum foo.json:

                      95% CI                99% CI               99.9% CI             99.99% CI
    sys time   [  -7.4% ..   +3.5%]  [  -9.1% ..   +5.2%]  [ -11.1% ..   +7.2%]  [ -12.7% ..   +8.9%]
    user time  [ +24.6% ..  +26.2%]  [ +24.4% ..  +26.5%]  [ +24.1% ..  +26.8%]  [ +23.8% ..  +27.0%]
    wall time  [ +21.9% ..  +22.9%]  [ +21.8% ..  +23.0%]  [ +21.6% ..  +23.2%]  [ +21.5% ..  +23.3%]

sha1sum foo.json vs sha256sum foo.json:

                      95% CI                99% CI               99.9% CI             99.99% CI
    sys time   [  -0.2% ..  +11.0%]  [  -2.0% ..  +12.8%]  [  -4.1% ..  +14.8%]  [  -5.8% ..  +16.6%]
    user time  [+136.7% .. +138.3%]  [+136.4% .. +138.6%]  [+136.1% .. +138.9%]  [+135.9% .. +139.1%]
    wall time  [+125.5% .. +127.0%]  [+125.3% .. +127.2%]  [+125.0% .. +127.4%]  [+124.8% .. +127.7%]
```

You can even pipe the output of `cbdr sample` into `cbdr analyze` to see
the confidence intervals change as they're updated by new data:

<img src=https://github.com/asayers/cbdr/raw/master/demo.gif>

## Interpreting the results

Let's look at the table comparing md5 to sha1, and choose a p-value of 99%.
Looking at the wall-clock time, it indeed looks to be about 22% slower.
That said, you should avoid the temptation reduce the results to a single
number.  The statistically responsible way to report this benchmark to your
colleagues would be like this:

> sha1sum was 21.8-23% slower than md5sum by wall-clock (p=99%)

By contrast, looking at the system time, we see that the difference is between
-9.1% and +5.2%.  Because the confidence interval contains 0%, we don't have
enough evidence to say that the time spent in the kernel is any different.

> The difference in system time was within noise (p=99%)
