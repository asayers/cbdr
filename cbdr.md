# CBDR

This repo contains a suite of tools called `cbdr`.

`cbdr sample` takes a list of benchmarks in the form `name:program`.
It randomly selects a benchmark, runs the program, reports the execution
time, and loops.  The output is CSV-formatted and goes on forever, so lets
limit it with `head` and format it with `column`.

```
$ cbdr sample "md5:md5sum $BIG_FILE" "sha1:sha1sum $BIG_FILE" "sha256:sha256sum $BIG_FILE" | head | column -ts,
Warming up md5...
Warming up sha1...
Warming up sha256...

benchmark  sys time  user time  wall time
md5        0.01      0.12       0.131099686
sha1       0.03      0.13       0.155063893
sha256     0.02      0.32       0.344348186
sha256     0.01      0.32       0.335235973
md5        0         0.13       0.128056813
md5        0.01      0.13       0.130115718
md5        0.02      0.1        0.131369468
sha1       0         0.15       0.149611563
md5        0.01      0.12       0.128339435
```

`cbdr analyze` takes the output of `cbdr sample` and summarizes the differences
between the benchmarks.

```
$ cbdr analyze <results.csv
           md5            sha1           difference (99.9% CI)
sys time   0.011 ± 0.007  0.011 ± 0.008  [ -14.7% ..  +17.5%]
user time  0.116 ± 0.009  0.138 ± 0.009  [ +17.0% ..  +20.6%]
wall time  0.128 ± 0.006  0.149 ± 0.006  [ +15.9% ..  +18.0%]
samples    392            410

           sha1           sha256         difference (99.9% CI)
sys time   0.011 ± 0.008  0.012 ± 0.008  [ -12.0% ..  +20.0%]
user time  0.138 ± 0.009  0.334 ± 0.013  [+139.6% .. +143.4%]
wall time  0.149 ± 0.006  0.345 ± 0.011  [+129.8% .. +132.5%]
samples    410            422
```

You can even pipe the output of `cbdr sample` into `cbdr analyze` to see
the confidence intervals change live as they're updated by new data.

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
