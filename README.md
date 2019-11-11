# Continuous Benchmarking, Done Right

Continuous integration (which normally means "running the tests on every
commit") has become standard practise, and for good reason: if you don't
have good tests and run them regularly, you're bound to allow semantic
regressions (aka "bugs") into your code.  Continuous benchmarking is likewise
essential if you don't want to allow performance regressions to happen;
yet it's far less commonplace.

This is simply because it's harder: tests are deterministic, whereas benchmarks
are not.  Even "kinda deterministic" proxies such as instruction count are
quite variable in practice.  The property you care about (be it wall-time,
max RSS, or whatever) is not a number but is in fact a distribution, and the
property you _really_ care about is the mean of that distribution; and the
thing you _actually really_ care about is how much that mean changes when
you apply a particular patch.

Without further ado, here's my method:

1. Pick a pair of commits you care about
2. Pick one at random and benchmark it, appending the results to your set of measurements
3. Compute the 99% confidence interval for the diffence of the means
4. If the confidence interval is smaller than the smallest regression you
   care about, you're done.  If not, go to step 2.

If your benchmark produces multiple values (eg. wall time and max RSS),
then you want to check that none of them have regressed.  This multiplies
your chance of a false positive by the number of values involved.  You can
counteract this by multiplying the widths of your CIs by the same number.

There's some stuff in this repo which might help you implement a scheme like
the one I described, but to be honest it's not that hard and the setup will
depend a lot on how your benchmarks look.

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
in the winter?

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
