# CBDR

This repo contains a suite of tools called `cbdr`.  Here's an example comparing
`cargo check` to `cargo test`:

```
cbdr run 'cargo c' 'cargo t'
```

Here's what it looks like:

<img src=https://github.com/asayers/cbdr/raw/master/demo.gif>

You can also save the raw benchmark results with `cbdr run --stdout`
or `cbdr run --out=<FILE>`, and then post-process them with `cbdr
{summaize,diff,pretty}`.

## Interpreting the results

Let's say we want to find out how much slower sha1sum is than mdf5sum.  We run this command:

```
cbdr run --bench=bench.sh "md5sum capture.pcapng.xz" "sha1sum capture.pcapng.xz"
```

And get the following output:

```
Warming up md5sum capture.pcapng.xz...
Warming up sha1sum capture.pcapng.xz...

md5sum capture.pcapng.xz..sha1sum capture.pcapng.xz:
                       -99%      -95%         Δ      +95%      +99%
    max_rss:    [   -2.281%   -1.799%   -0.297%   +1.205%   +1.687% ]  (2033.129 -> 2027.086)
    wall_time:  [  +26.723%  +27.540%  +30.089%  +32.637%  +33.454% ]  (0.756 -> 0.983)
```

Looking at the wall-clock time, it indeed looks to be about 30% slower.
That said, you should avoid the temptation reduce the results to a single
number.  The statistically responsible way to report this benchmark to your
colleagues would be like this:

> sha1sum was 27.5%-32.6% slower than md5sum (p=95%)

By contrast, looking at the "max_rss" row, we see that the difference is
between -1.8% and +1.2%.  This means that we don't have enough evidence to
support the idea that the memory usage is different, one way or the other.

> Memory usage was within noise (p=95%)

In case you're wondering, the bench.sh script above looks like this:

```
out=$(mktemp) && /usr/bin/time -o$out -f'{ "wall_time": %e, "max_rss": %M }' $@ &>/dev/null && cat $out
```

Of course, you could measure other things instead.  Here's a run using
`perf stat`:

```
                              -99%      -95%         Δ      +95%      +99%
    branch_misses:     [  +49.745%  +50.398%  +52.433%  +54.468%  +55.121% ]  (139978.000 -> 213372.333)
    branches:          [ +560.242% +560.259% +560.312% +560.366% +560.383% ]  (22268445.685 -> 147041291.940)
    context_switches:  [   -4.609%  +10.518%  +57.661% +104.805% +119.932% ]  (2.696 -> 4.250)
    cpu_migrations:    [ -326.871% -191.738% +228.571% +648.880% +784.014% ]  (0.011 -> 0.036)
    cpu_utilization:   [   -0.006%   -0.002%   +0.010%   +0.021%   +0.025% ]  (1.000 -> 1.000)
    cycles:            [  +17.691%  +17.730%  +17.850%  +17.971%  +18.010% ]  (2794011079.902 -> 3292753851.464)
    instructions:      [ +137.573% +137.574% +137.575% +137.576% +137.577% ]  (4584444365.533 -> 10891491773.631)
    page_faults:       [   -0.672%   -0.384%   +0.514%   +1.413%   +1.701% ]  (75.054 -> 75.440)
    task_clock:        [  +29.378%  +29.623%  +30.380%  +31.138%  +31.383% ]  (729.043 -> 950.529)
```

Notice that cycles and instruction count are relatively stable (ie. have
tight CIs) compared to task_clock, which is why people like them as a proxy.
They're only loosely correlated with wall time, however.
