# CBDR

This repo contains a suite of tools called `cbdr`.  `cbdr sample` repeatedly
selects a program at random from a list and benchmarks it.

```
cbdr sample 'md5sum foo.json' 'sha1sum foo.json' 'sha256sum foo.json' | head | column -s, -t
Warming up md5sum foo.json...
Warming up sha1sum foo.json...
Warming up sha256sum foo.json...
target              sys_time  user_time  wall_time
md5sum foo.json     0.01      0.03       0.026581489
sha256sum foo.json  0         0.07       0.072450366
sha1sum foo.json    0         0.04       0.041179884
md5sum foo.json     0         0.02       0.025899522
md5sum foo.json     0.01      0.02       0.026360146
sha256sum foo.json  0         0.07       0.071064774
md5sum foo.json     0         0.02       0.026017469
sha1sum foo.json    0         0.03       0.031259894
md5sum foo.json     0.01      0.03       0.027783781
```

Pipe the output into `cbdr analyze` to see a (live-updating) summary of the
differences between the benchmarked programs.

```
cbdr sample 'md5sum foo.json' 'sha1sum foo.json' 'sha256sum foo.json' | cbdr analyze
Warming up md5sum foo.json...
Warming up sha1sum foo.json...
Warming up sha256sum foo.json...

md5sum foo.json..sha1sum foo.json:
               -99%       -95%       Δ         +95%      +99%      before  after  ratio
    sys_time   -155.477%  -124.892%  -33.908%  +57.076%  +87.660%  0.003   0.002  0.661
    user_time  +5.935%    +9.853%    +21.609%  +33.366%  +37.283%  0.026   0.032  1.216
    wall_time  +13.302%   +14.955%   +19.846%  +24.737%  +26.390%  0.029   0.034  1.198

sha1sum foo.json..sha256sum foo.json:
               -99%       -95%       Δ          +95%       +99%       before  after  ratio
    sys_time   -103.011%  -57.591%   +78.462%   +214.514%  +259.934%  0.002   0.003  1.785
    user_time  +124.173%  +127.527%  +137.625%  +147.724%  +151.078%  0.032   0.075  2.376
    wall_time  +120.274%  +121.936%  +126.942%  +131.948%  +133.610%  0.034   0.078  2.269
```

## Interpreting the results

Let's look at the table comparing mdf5 to sha1.  Looking at the wall-clock
time, it indeed looks to be about 20% slower.  That said, you should avoid
the temptation reduce the results to a single number.  The statistically
responsible way to report this benchmark to your colleagues would be like this:

> sha1sum was 15-25% slower than md5sum by wall-clock (p=95%)

By contrast, looking at the system time, we see that the difference is between
-125% and +57%.  Because the confidence interval contains 0%, we don't have
enough evidence to say that the time spent in the kernel is any different.

> The difference in system time was within noise (p=95%)
