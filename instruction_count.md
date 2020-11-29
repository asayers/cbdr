This is an archived copy of [https://hackmd.io/sH315lO2RuicY-SEt7ynGA?view],
written by eddyb from the rustc team.

---

# Hardware performance counter support (via `rdpmc`)

## Pull Requests

This report is a companion to these GitHub PRs:
* [`rust-lang/measureme#143`](https://github.com/rust-lang/measureme/pull/143) for the bulk of the implementation
* [`rust-lang/rust#78781`](https://github.com/rust-lang/rust/pull/78781) for integration into `rustc`
    * adds a `-Z self-profile-counter` flag to choose from `measureme`'s named counters

Some details are only found in those PRs, while the rest is below.

## Motivation

We were attempting to gather detailed information about the impact of Rust PRs on the performance of specific parts of the compiler, such as the trait system.
The `-Z self-profile` data, however, was far too noisy for this, due to its reliance on measuring time, which factors in far more about the system, and it would be an incredibly uphill battle to try to make the results more reliable, that everyone would have to replicate in order to get the same effect.
You can see the noise in the `-Z self-profile` data here (third graph): https://github.com/rust-lang/rustc-perf/pull/661 - it's several time larger than the actual changes we wanted to see.

Hardware performance counters offer, in theory, an out: the CPU itself can accurately keep track of events like "Instructions retired", and even allow the OS to isolate individual userspace threads from anything else running on the same machine, including the kernel itself.
`perf.rust-lang.org` already tracks such counters, via `perf stat`, but there an entire process is measured, as opposed to the query-level granularity of `-Z self-profile`.

While tracking *only* the number of executed instructions might hide certain effects, the long-term plan is to experiment with tapping into some parts of the cache hierarchy (likely L1d and L2) and using cache misses as a proxy for the cost of non-locality.

## Results

### Overhead

To determine the cost of sampling the instruction counter (via `rdpmc`), as opposed to measuring time (via `std::time::Instant`, which uses `clock_gettime(CLOCK_MONOTONIC)`), the total number of instructions was tracked while compiling libcore in "check mode".

By gathering groups of 10 runs per counter type (including a "zero" counter that always produces the `0` constant, for the baseline), and dividing the difference between counter types' totals by the number of counter reads, we can reliably know the per-read cost (in the number of instructions).

Note that the totals are measured by `instructions-minus-irqs:u`, regardless of which counter is being used by the profiler, so the noise here is not indicative of the *quality* of the profile data, but rather the determinism of the whole execution (including sampling the counter used by the profiler).

More details, and a revision history, are available in [a gist](https://gist.github.com/eddyb/7a0a55411441142765db6cfa41504e50), but these are the final results:

|<small>Counter</small>|<small>Total</small><br><sup>`instructions-minus-irqs:u`</sup>|<small>Overhead<br>from "Baseline"</small><br><sup>(for all 1903881<br>counter reads)</sup>|<small>Overhead<br>from "Baseline"</small><br><sup>(per each<br>counter read)</sup>|
|-|-|-|-|
|Baseline|63637621286 ±6|||
|<sub>`instructions:u`</sub>|63658815885 ±2|&nbsp;&nbsp;+21194599 ±8|&nbsp;&nbsp;+11|
|<sub>`instructions-minus-irqs:u`</sub>|63680307361 ±13|&nbsp;&nbsp;+42686075 ±19|&nbsp;&nbsp;+22|
|<sub>`wall-time`</sub>|63951958376 ±10275|+314337090 ±10281|+165|

From this we can gather that counting instructions, even when subtracting IRQs, can be an order of magnitude faster than measuring time (though this may vary depending on the actual instructions executed).

You may also notice measuring time is non-deterministic in the total number of instructions (around ±5 instructions for every 1000 `clock_gettime(CLOCK_MONOTONIC)` calls, though keep in mind we don't know the distribution), likely due to the synchronization mechanism between libc and the kernel requiring an extra loop iteration.

### "Macro" noise (self time)

Using the profile data from the runs above (ignoring the "Baseline"), we've looked at the 10 largest queries, and variance in the "Self time" (or "Self count") between runs, for each counter:

<small>`wall-time` (ns)</small> | <small>`instructions:u`</small> | <small>`instructions-minus-irqs:u`</small>
-: | -: | -:
||<sub>`typeck`</sub>
5478261360 ±283933373 (±~5.2%) | 17350144522 ±6392 (±~0.00004%) | 17351035832.5 ±4.5 (±~0.00000003%)
||<sub>`expand_crate`</sub>
2342096719 ±110465856 (±~4.7%) | 8263777916 ±2937 (±~0.00004%) | 8263708389 ±0 (±~0%)
||<sub>`mir_borrowck`</sub>
2216149671 ±119458444 (±~5.4%) | 8340920100 ±2794 (±~0.00003%) | 8341613983.5 ±2.5 (±~0.00000003%)
||<sub>`mir_built`</sub>
1269059734 ±91514604 (±~7.2%) | 4454959122 ±1618 (±~0.00004%) | 4455303811 ±1 (±~0.00000002%)
||<sub>`resolve_crate`</sub>
942154987.5 ±53068423.5 (±~5.6%) | 3951197709 ±39 (±~0.000001%) | 3951196865 ±0 (±~0%)
||<sub>`hir_lowering`</sub>
530050063.5 ±30695921.5 (±~5.8%) | 1381364181 ±22 (±~0.000002%) | 1381363724 ±0 (±~0%)
||<sub>`build_hir_map`</sub>
367098179.5 ±18410803.5 (±~5%) | 2099522033.5 ±12.5 (±~0.0000006%) | 2099521684 ±0 (±~0%)
||<sub>`check_mod_item_types`</sub>
354434450.5 ±21655950.5 (±~6.1%) | 898871840.5 ±21.5 (±~0.000002%) | 899018351.5 ±0.5 (±~0.00000006%)
||<sub>`check_impl_item_well_formed`</sub>
323550938.5 ±16711421.5 (±~5.2%) | 805382346 ±42 (±~0.000005%) | 805841502 ±1 (±~0.0000001%)
||<sub>`check_item_well_formed`</sub>
321212644.5 ±18409991.5 (±~5.7%) | 898599944 ±22 (±~0.000002%) | 899122104.5 ±1.5 (±~0.0000002%)

From this, the noise reduction becomes pretty clear, as even the worst case (thanks to unpredictable IRQs) for instruction-counting is 100,000 times (i.e. 5 orders of magnitude) smaller than the 5% `wall-time` noise.

See [Caveats/Subtracting IRQs](#Subtracting-IRQs) below for why `instructions-minus-irqs:u` is much better but still not perfect.

### "Micro" noise (individual sampling intervals)

Using [`summarize aggregate`](https://github.com/rust-lang/measureme/pull/129) (see also [an explanation](#AMD-patented-time-travel-and-dubbed-it-SpecLockMapnbspnbspnbspnbspnbspnbspnbspnbspor-“how-we-accidentally-unlocked-rr-on-AMD-Zen”) below), on the same profile data again, we get these smallest and largest 5 variances (for "sampling intervals", i.e. distance between consecutive pairs of counter reads):

* `wall-time`
```
  ±0 ns: 44 occurrences, or 0.00%
  ±4.5 ns: `hir_owner()`
  ±5 ns: 10085 occurrences, or 0.53%
  ±5.5 ns: 99 occurrences, or 0.01%
  ±9.5 ns: 18 occurrences, or 0.00%
...
  ±16510104 ns: in `typeck()`, between `parent_module_from_def_id()` and `upvars_mentioned()`
  ±18410803.5 ns: `build_hir_map()`
  ±30695921.5 ns: `hir_lowering()`
  ±53068423.5 ns: `resolve_crate()`
  ±106282082.5 ns: in `expand_crate()`, after `pre_AST_expansion_lint_checks()`
```

* `instructions:u` 
```
  ±0 instructions: 1830563 occurrences, or 96.15%
  ±0.5 instructions: 69126 occurrences, or 3.63%
  ±1 instructions: 2466 occurrences, or 0.13%
  ±1.5 instructions: 715 occurrences, or 0.04%
  ±2 instructions: 219 occurrences, or 0.01%
...
  ±175.5 instructions: in `typeck()`, between `parent_module_from_def_id()` and `upvars_mentioned()`
  ±214 instructions: `free_global_ctxt()`
  ±226 instructions: in `typeck()`, between `opt_const_param_of()` and `in_scope_traits_map()`
  ±238.5 instructions: in `typeck()`, between `opt_const_param_of()` and `in_scope_traits_map()`
  ±2278.5 instructions: in `expand_crate()`, after `pre_AST_expansion_lint_checks()`
```

* `instructions-minus-irqs:u`
```
  ±0 instructions: 1903452 occurrences, or 99.98%
  ±0.5 instructions: 423 occurrences, or 0.02%
  ±1 instructions: 3 occurrences, or 0.00%
  ±4.5 instructions: `write_crate_metadata()`
```

## Results <small>(for `polkadot-runtime-common`)</small>

These are similar to the previous "Results" section, but instead of libcore, they're for the `polkadot-runtime-common` crate, at [tag `v0.8.26`](https://github.com/paritytech/polkadot/tree/v0.8.26) (also in "check mode").

*See [Caveats/Non-deterministic proc macros](#Non-deterministic-proc-macros) below for an ASLR-like effect we encountered here (but not for libcore, as it uses no proc macros), and how we worked around it.*

### "Macro" noise (self time) <small>(for `polkadot-runtime-common`)</small>

<small>`wall-time` (ns)</small> | <small>`instructions:u`</small> | <small>`instructions-minus-irqs:u`</small>
-: | -: | -:
||<sub>`expand_crate`</sub>
1255640143 ±62610049 (±~5%) | 3646160968.5 ±1940.5 (±~0.00005%) | 3646160240 ±0 (±~0%)
||<sub>`typeck`</sub>
678068989 ±39533817 (±~5.8%) | 1664414250.5 ±1110.5 (±~0.00007%) | 1664595300.5 ±2.5 (±~0.0000002%)
||<sub>`mir_borrowck`</sub>
286718041 ±19533032 (±~6.8%) | 837858656 ±502 (±~0.00006%) | 837973316 ±1 (±~0.0000001%)
||<sub>`metadata_register_crate`</sub>
131774774 ±11642890 (±~8.8%) | 537739743.5 ±203.5 (±~0.00004%) | 537746710 ±0 (±~0%)
||<sub>`evaluate_obligation`</sub>
98972055 ±5956386 (±~6%) | 245732462.5 ±184.5 (±~0.00008%) | 245852905.5 ±2.5 (±~0.000001%)
||<sub>`resolve_crate`</sub>
95908303.5 ±8120924.5 (±~8.5%) | 281321111 ±108 (±~0.00004%) | 281329454 ±0 (±~0%)
||<sub>`mir_built`</sub>
91490959.5 ±6749247.5 (±~7.4%) | 255743130.5 ±160.5 (±~0.00006%) | 255789874.5 ±1.5 (±~0.0000006%)
||<sub>`type_op_prove_predicate`</sub>
70004550.5 ±5436502.5 (±~7.8%) | 175309312.5 ±128.5 (±~0.00007%) | 175357244 ±0 (±~0%)
||<sub>`check_item_well_formed`</sub>
50069210.5 ±4185722.5 (±~8.4%) | 144582558 ±83 (±~0.00006%) | 144642844 ±1 (±~0.0000007%)
||<sub>`check_impl_item_well_formed`</sub>
40564784.5 ±2546996.5 (±~6.3%) | 107322787.5 ±82.5 (±~0.00008%) | 107359747.5 ±0.5 (±~0.0000005%)

### "Micro" noise (individual sampling intervals) <small>(for `polkadot-runtime-common`)</small>

* `wall-time`
```
  ±0 ns: 228 occurrences, or 0.03%
  ±0.5 ns: 4 occurrences, or 0.00%
  ±4.5 ns: in `implementations_of_trait()`, after `metadata_decode_entry_implementations_of_trait()`
  ±5 ns: 13744 occurrences, or 1.63%
  ±5.5 ns: 119 occurrences, or 0.01%
...
  ±4402505.5 ns: `free_global_ctxt()`
  ±8120924.5 ns: `resolve_crate()`
  ±10358487 ns: in `expand_crate()`, between `metadata_load_macro()` and `metadata_load_macro()`
  ±11484159 ns: in `expand_crate()`, between `metadata_load_macro()` and `metadata_load_macro()`
  ±44234813 ns: in `expand_crate()`, between `metadata_load_macro()` and `metadata_load_macro()`
```

* `instructions:u` 
```
  ±0 instructions: 828814 occurrences, or 98.00%
  ±0.5 instructions: 15484 occurrences, or 1.83%
  ±1 instructions: 899 occurrences, or 0.11%
  ±1.5 instructions: 291 occurrences, or 0.03%
  ±2 instructions: 63 occurrences, or 0.01%
...
  ±125.5 instructions: in `expand_crate()`, between `metadata_load_macro()` and `metadata_load_macro()`
  ±169 instructions: between `codegen_crate()` and `free_global_ctxt()`
  ±242 instructions: `free_global_ctxt()`
  ±1428 instructions: in `expand_crate()`, between `metadata_load_macro()` and `metadata_load_macro()`
  ±7588.5 instructions: `self_profile_alloc_query_strings()`
```

* `instructions-minus-irqs:u`
```
  ±0 instructions: 845456 occurrences, or 99.97%
  ±0.5 instructions: 229 occurrences, or 0.03%
  ±1 instructions: 3 occurrences, or 0.00%
  ±4.5 instructions: `write_crate_metadata()`
  ±5 instructions: in `link()`, between `finish_ongoing_codegen()` and `llvm_dump_timing_file()`
  ±18 instructions: in `link()`, after `link_crate()`
  ±20 instructions: in `codegen_crate()`, between `assert_dep_graph()` and `serialize_dep_graph()`
  ±188.5 instructions: between `codegen_crate()` and `free_global_ctxt()`
  ±332 instructions: `free_global_ctxt()`
  ±9430.5 instructions: `self_profile_alloc_query_strings()`
```

One thing to note here is that even though `instructions-minus-irqs:u` is worse here than it was for libcore, all of the extra noise is contained in IO (during/after writing the `.rmeta` file).
And because it's *isolated* in those parts of the compiler, it has no effect whatsoever on queries.

## Caveats

### Disabling ASLR

*See also [Challenges/ASLR: it’s free entropy](#ASLR-it’s-free-entropy) for some backstory and more details.*

In order to get the best results, disabling ASLR by e.g. running `rustc` under `setarch -R` is necessary.

Note that the `rustup` wrapper binary which allows `rustc +foo ...` does not currently appear to propagate ASLR being disabled to the actual `rustc` binary (e.g. `~/.rustup/toolchains/.../bin/rustc`), so you will need to refer to it directly, for now.

Long-term we may want to make `rustc`'s use of pointer hashing more intrinsically deterministic, by using special arenas which prefer known addresses, instead of letting the OS pick them.

### Non-deterministic proc macros

When profiling a `rustc` compilation which uses proc macros, they may contain arbitrary user code, including sources of randomness and IO. The easiest way this can impact the rest of `rustc` is through introducing ASLR-like effects with non-deterministic allocation sizes.

For example, the very-widely-used `serde_derive` proc macro happens to use `HashSet`, which by default uses a randomized hash (just like `HashMap`), and that will result in different (re)allocation patterns of the `HashSet` heap data, effectivelly randomizing any heap addresses allocated afterwards (by other parts of `rustc`).

In order to fully eliminate ASLR-like effects due to the use of `HashMap`/`HashSet` in proc macros, during testing and data collection, we had to temporarily change the behavior of `std::collections::hash_map::RandomState` to *not* use the `getrandom` syscall.

Long-term we'd want either `rustc`/`proc_macro` to hook `std` and disable `getrandom` for `HashMap`/`HashSet`, *or* some sort of tool that intercepts the `getrandom` syscall using `ptrace` and/or SECCOMP filters. Alternatively/additionally, we could try getting `std` to use the glibc wrapper when available, making it possible to use `LD_PRELOAD` to defuse `getrandom`.

### Subtracting IRQs

*See also [Challenges/Getting constantly interrupted](#Getting-constantly-interrupted) for some backstory and more details.*

`instructions-minus-irqs:u` exists to account for/work around overcounting (in `instructions:u`) associated with interrupts.

Our hypothesis is that the `iret` instruction, which the kernel uses to return execution from an interrupt handler, back to the (interrupted) userspace code, is counted as "retiring in userspace".

While this (if true) would be pedantically/technically correct, it's practically undesirable, as the otherwise userspace-only count is now tainted by 1 extra instruction per (unpredictable) interrupt.

Additionally, subtracting IRQs requires CPU family/model detection logic to determine the correct configuration for the IRQs counter.
This is the most ad-hoc part of the entire `measureme` PR, and some of it relies on undocumented/redacted-out information (verified through testing) - but also, we later learned that projects like `rr` do something similar, long-term we probably want to split this into a general-purpose library.

`instructions-minus-irqs:u` is still not perfect as it requires two `rdpmc`s, and interrupts can come in between them, but the resulting noise is usually limited to 1-2 instructions in the "total count" for any `measureme` event, and relatively rare, so much better than the interrupt noise it's removing.

There [is a way (`CONFIG_NO_HZ_FULL`)](https://www.kernel.org/doc/Documentation/timers/NO_HZ.txt) to get the Linux kernel to avoid using timers on some cores, which would further decrease the noise (or perhaps even allow using `instructions:u` directly), but we've decided against experimenting with it, as it impacts scheduling system-wide and so would require a dedicated machine for profiling - the [documentation](https://www.kernel.org/doc/Documentation/timers/NO_HZ.txt) even says:
> Unless you are running realtime applications or certain types of HPC workloads, you will normally -not- want this option.

### Lack of support for multiple threads

The initial implementation doesn't support reading the counter from threads other than the original one it was created on. `rdpmc` will be attempted, but no useful value can be read, as the counter is only running for the original thread.

This works for `cargo check` (i.e. `rustc --emit=metadata`) compilations, and those were our main focus, as `-Z self-profile` is most fine-grained (i.e. at the query level) in the "check" part of the compilation. It could be possible to measure LLVM as well using `-Z no-llvm-threads`, but we haven't confirmed this works reliably.

Additionally, it's harder to get determinstic and comparable results if multiple threads are involved, though queries themselves could remain pretty stable (except for unpredictable allocation in other threads inducing an ASLR-like effect).

Long-term, we should try to do (at least) one of:
* refactor `rustc` to hold one "profiler handle" (with its own counter) per-thread
* use a `Vec<Counter>` in `measureme`, and index it by `std::thread::current().id()`
    * or rather, the thread index passed to `Profiler` methods by `rustc`
* enable `inherit` when creating the counter with `perf_event_open`
    * this might not do anything useful for `measureme`, further experimentation needed
    * if it *does* work, it would be the most convenient way

## Challenges

### How do we even read hardware performance counters?

The Linux API for interfacing with both "software events" (tracked manually in the kernel) and hardware performance counters, is `perf_event_open`, which returns a file descriptor for a pseudo-file that can then be read from to sample the counter.

However, we wanted to avoid the cost of a `read` syscall, especially as x86 has the `rdpmc` instruction, and Linux claims to allow it to be used in userspace, once `perf_event_open` sets up a counter.
The information necessary for using `rdpmc` is documented as being available by `mmap`-ing the pseudo-file `perf_event_open` returns, creating a region of shared memory (which can also be used as a ring buffer for software events, sampling profiling, etc. - though none of that is relevant here).

We started out by looking at examples which used a combination of `perf_event_open`, `mmap` and `asm("rdpmc")`, and while there was some variation (such as also keeping track of time), they all seemed to share this rough structure:
```c
p = mmap(perf_event_open(...));

// Every time you want to read the counter:
do {
    // The kernel increments `lock` when it changes anything, hence
    // the do-while loop to retry if the "sequence number" changed.
    seq = p->lock;
    barrier(); // Avoids reading stale values below.

    // Read the counter (not shown here: the `rdpmc` function 
    // using `asm("rdpmc")`) and add a kernel-supplied "offset".
    count = rdpmc(p->index - 1) + p->offset;

    // Sign-extend from `p->pmc_width` bits to 64 bits.
    shift = 64 - p->pmc_width;
    count = count << shift >> shift;

    barrier(); // Avoids reading stale `lock` in condition below.
} while(p->lock != seq);
```
You can see [an example of this in `deater/perf_event_tests`](
https://github.com/deater/perf_event_tests/blob/a379ad31192c0780278507b0acefe70d872fb643/tests/rdpmc/rdpmc_lib.c#L81-L180) (with several copies of that function existing across the repository).
The Linux kernel doc comments [have a simpler example](https://github.com/torvalds/linux/blob/ed8780e3f2ecc82645342d070c6b4e530532e680/include/uapi/linux/perf_event.h#L489-L523) with the sign-extension aspect [documented separately](https://github.com/torvalds/linux/blob/ed8780e3f2ecc82645342d070c6b4e530532e680/include/uapi/linux/perf_event.h#L543-L551).

There are a few downsides to this "standard" approach:
* the synchronization mechanism seems somewhat sketchy, at least for Rust (unsynchronized reads + manual barriers, instead of atomic reads)
* if the kernel *does* update any of the fields, there is non-determinism introduced, as the loop will execute more than once (and therefore count more instructions)
* for the fields that *aren't* updated (like `index`), or which may be constant (like `pmc_width`), treating them like they could change is less efficient than it needs to be

Taking a closer look at each of the 3 fields:
* `index`: can't avoid reading it at least once (e.g. if we're running under `perf stat -e ...`, we wouldn't get the first counter)
    * as long as we don't stop the counter, its `index` shouldn't change, so we could keep our own copy of it
* `offset`: this was the most worrying, as the documentation makes it seem like it could be updated by the kernel during e.g. context switching
    * in practice, it seemed to remain at 2<sup>pmc_width-1</sup> - 1
    * the specific value would suggest it's a "bias", so that counters start in the middle of their range, as opposed to around 0
        * this is presumably related to the ability to wait for the counter to increment by a certain amount, detected through the counter overflowing, but thankfully we don't need any of that
    * additionally, as we wanted to subtract some initial counter value to get a relative count, the `offset` field could be completely ignored (i.e. `(b + offset) - (a + offset)` is just `b - a`)
* `pmc_width`: it seems possible that different CPUs may have different physical counter widths (up to the 64 bits `rdpmc` can provide)
    * on every x86 CPU we've tested, it was always 48
    * we could therefore check it's what we expect at the start, and hardcode it for the reads themselves

In the end, we were able to avoid accessing any of the shared memory and dealing with the associated synchronization, when reading the counter, but kept the synchronization loop for the initial setup.

### ASLR: it's free entropy

From previous experience, we were aware of the impact ASLR ("Address Space Layout Randomization", i.e. placing the program, and its stack, heap, etc. at random addresses, as a security measure) can have on `rustc`'s instruction-level determinism.

While `rustc`'s external outputs should never change on repeated executions, some internal data structures may change performance characteristics due to `rustc`'s reliance on using pointer addresses (after having deduplicated the data being referenced) for faster equality and hashing, such as when used as `HashMap` keys.

In order to run e.g. some program `./foo`, with ASLR disabled, on Linux, one can use the `setarch -R ./foo ...` command (or `setarch x86_64 -R ./foo ...` in older versions), where `-R` specifically is the flag for disabling ASLR.
Going this route avoids needing root access, and won't affect the rest of the system.
It can also be done with `personality(ADDR_NO_RANDOMIZE)`, if you already have a `rustc` wrapper, but for manual testing `setarch -R` works great.

We did run into a problem, however, where `rustup`'s wrapper commands that allow picking toolchains (e.g. `rustc +nightly ...` vs `rustc +stable ...`) don't seem to propagate the personality flags (when ran as `setarch -R rustc +local ...`), so for a while disabling ASLR didn't appear to have an effect.
The solution was to run the locally-built `rustc` directly, i.e. `setarch -R ./build/x86_64-unknown-linux-gnu/stage1/bin/rustc`, which did show a significant difference.

With ASLR enabled, the variance in the totals (43 billion at the time) was around ±2.5 million, but disabling ASLR brought it to under ±10k (which we were later able to also eliminate).

That makes "non-deterministic addresses" (whether from ASLR or something else) by far the largest source of noise we've observed, and we're able to rely on the large magnitude to tell it apart.

One example of such an effect produced without ASLR itself, was a correlation between the length of PIDs (in base 10), and the total number of instructions, which we accidentally noticed across a set of 100 runs:
![image](https://user-images.githubusercontent.com/77424/97382331-cbb90b00-18d3-11eb-8414-727cbfe09d2c.png)

That turned out to be from `format!("{}-{}", crate_name, std::process::id())`, used by `rustc -Z self-profile` as the prefix for the profile data file(s). The size of that single allocation was able to act as a (very weak) pseudorandom form of ASLR, after amplifying the deviation, through cloning and concatenation.

Our workaround was to pad the PID with zeros (up to 6 digits), but similar sources of non-deterministic allocation sizes could still be hiding elsewhere, and we will have to address them as they show up in the data.

### The serializing instruction

Both Intel and AMD warn about using the `rdpmc` instruction without some sort of "serializing instruction", and how it could lead to inaccurate values being read out.

In this context, "serialized execution" refers to executing instructions "serially", i.e. in the order they exist in memory, as opposed to the CPU reordering them based on dependencies between them and available hardware resources.

As a more concrete example, for the "Instructions retired" counter, `rdpmc` may show a lower value as a few earlier instructions might not have "retired" yet, by the time `rdpmc` executed.

While not making `rdpmc` self-serializing is a counterproductive design decision, it does simplify the CPU (and as we found out later, this is a bit of a theme).

Coming back to the `perf_event_open` + `mmap` + `asm("rdpmc")` examples we found, none of them were using a "serializing instruction", so that's another downside to add, on top of the synchronized loop.

As Intel and AMD do not entirely agree on which instructions count as "fully serializing", we had to go with the clunkier `cpuid` (which both Intel and AMD document as "fully serializing"), even if we experimentally saw that e.g. `lfence` has a similar noise-reducing effect on AMD Zen (an effect which is likely only enabled as part of mitigations for speculative execution vulnerabilities such as Spectre).
This was before we developed the `summarize aggregate` tool, so we had limited insight into fine-grained noise, but the difference between serialized and unserialized was visible in the variance of per-query statistics, i.e. in the `summarize summarize --json` data.

There are [news of a possible `serialize` instruction](https://www.phoronix.com/scan.php?page=news_item&px=Linux-SERIALIZE-Sync-Core) which does exactly what's needed, and no more, but it looks like using it would require CPU detection and dynamically picking it (which would also allow using `lfence`, where happens to also be fully serializing).

Out of all the noise prevention measures we've taken, this one has the least impact, as there aren't that many instruction reordering opportunities just before the `rdpmc`, so it's more of a preventative measure, and we may end up relaxing it in the future.

### Getting constantly interrupted

By this point, we were getting variance (in the total number of instructions) anywhere between ±5k and ±10k, and started investigating it, as there didn't appear to be any obvious source for that noise.

In order the validate some of our basic assumptions, we combined the relevant `measureme` and some dependencies into a self-contained `rdpmc-bench.rs`, and had [the initial version](https://gist.github.com/eddyb/269491b0b59605f39f1d0a9dcf535c4a/d003c82d38c3ff0df18764acb8abd2a6e39fe180) measure trivial loops, printing the deviation from the expected number of instructions.

So for each of the measured loops it could show:
* negative values: undercounting (i.e. some instructions weren't counted)
* exactly `0`: perfect (and our original expectation)
* positive values: overcounting (i.e. either some instructions were counted more than once, or unrelated instructions were counted)

What we saw was *overcounting*: something else was getting into the counter, and it appeared to be proportional to the number of instructions being executed (but relatively small).

One of our early suspicions was that perhaps the context-switching associated with multithreading (especially on laptop CPUs with few cores) was failing to correctly save and restore the counters. We'd later learn that the counters are largely impervious to such problems, thanks to the hardware being able to automatically pause them when outside of user mode, but at the time we started looking at various metrics available through `perf stat`, such as "software events" or even Linux kernel "tracepoints".

Eventually we noticed that the total overcounting correlated strongly with the *time* the process was running for, even when that varied between different runs, and [**@m-ou-se**](https://github.com/m-ou-se) realized that on her machine it was ~300 extra instructions per second *and* her Linux kernel was configured with `CONFIG_HZ=300`.

We were then able to confirm that the overcounting did match `CONFIG_HZ` across several different machines (with different Linux distros and varying `CONFIG_HZ`). On some machines we were able to confirm the value by simply executing `zgrep CONFIG_HZ /proc/config.gz`, but on others the kernel configuration was only found under `/boot`.

So what is this `CONFIG_HZ`? On the average Linux system, it's the frequency at which a (per-core) periodic timer will fire an interrupt (or "IRQ"), letting the kernel perform scheduling of all the active threads, and other periodic bookkeeping.

Soon thereafter [**@m-ou-se**](https://github.com/m-ou-se) found [a paper on non-determinism in hardware performance counters](http://web.eece.maine.edu/~vweaver/projects/deterministic/deterministic_counters.pdf), which pretty much confirmed the overcounting is directly tied to interrupts, and also offered a potential solution: subtract the number of interrupts, *assuming* you can count them.

One small note about the paper: it doesn't propose a mechanism for *how* the overcounting happens, and we ended up assuming it's the `iret` instruction the kernel uses to leave an interrupt handler and return to the code which was interrupted.
The reason for we went with that explanation is that an `iret` back to userspace *technically* retires in userspace - or in other words, the "Instructions retired" counter gets effectively unpaused *half-way* through executing `iret`, and extra silicon would be required to avoid that.
While it should be possible to confirm this *is* the case, with some custom baremetal code, or by getting the kernel itself to read the performance counter while in an interrupt handler (e.g. using the `read` syscall via the x86 `int 0x80` syscall interface, on a `perf_event_open` pseudo-file), we haven't done so yet.

Now that we knew how we might remove the interrupt noise, we started looking for a counter that would *at least* include the timer IRQs, and since `perf list` didn't seem to show anything of the sort on the machines we were looking at (Intel Ivy Bridge and AMD Zen 1 EPYC), we went to the (Intel and AMD) manuals, and found:
* Intel had `HW_INTERRUPTS.RECEIVED`, documented since Skylake as `01cb`
    * there was also a similar counter documented on some Atom microarchitectures, but given the lack of Atom test hardware, we decided against support them for now
* AMD had `LsIntTaken` (or "Interrupts Taken"), documented in two ways:
    * pre-Zen, it was `00cf` (going at least as far back as K8, the first AMD64 microarchitecture)
    * for Zen, it was `002c`
        * while it's missing from the older 1st gen EPYC documentation, which likely explains why `perf list` doesn't show it, it *is* documented nowadays

Despite Intel not documenting the counter on anything older than Skylake, we started trying it (via e.g. `perf stat -e r01cb:u`), and immediately confirmed that it works on Ivy Bridge.
Over the following days, we were able to also confirm it on Sandy Bridge (thanks to [**@alyssais**](https://github.com/alyssais)) and Haswell (thanks to [**@m-ou-se**](https://github.com/m-ou-se), [**@cuviper**](https://github.com/cuviper) and [**@nagisa**](https://github.com/nagisa).

We later noticed that [the aforementioned paper](http://web.eece.maine.edu/~vweaver/projects/deterministic/deterministic_counters.pdf) shows, on page 5, `HW_INTERRUPTS.RECEIVED` for Sandy Bridge and Ivy Bridge (and similar counters for even older CPUs, on both pages 4 and 5), leading us to conclude that Intel had "undocumented" these counters, instead of publishing errata and explaining the potential associated caveats.

Once we had the above information for the "hardware interrupts" (or "IRQs") counters we wanted to use, the `cpuid` instruction was used to determine which generation of CPU we're running on, whether we know of a counter to use, and which exact configuration is needed.

We then enhanced our `instructions:u` measurement to also `rdpmc` the IRQs counter, and subtract it, creating what we would later dub `instructions-minus-irqs:u` (to distinguish it from e.g. `perf stat -e instructions:u`, which doesn't account for IRQs at all).

For the `rdpmc-bench.rs` synthetic benchmark, after subtracting IRQs we stopped seeing overcounting at all, and instead had:
* some undercounting, on most machines, especially Intel Ivy Bridge and Haswell
    * perhaps these were the hyperthreading issues Intel "undocumented" the counters for?
* perfect results (`0` deviation from expected) on the AMD Zen 1 EPYC server
    * this machine may have an unfair advantage, i.e. its 48 hardware threads, 6-12x more than the Intel CPUs we looked at
    * long term we could look at intensive stress testing, using several times more userspace threads than hardware threads, to hunt down remaining non-determinism, but we haven't seen any evidence to suggest there could be any left

However, subtracting IRQs mostly didn't impact the (up to ±10k) noise we were seeing when compiling libcore, as the IRQ-driven overcounting stayed within a few hundred instructions between different runs, most of the time (though it did help with the occassional outlier).

And that's because...

### AMD patented time-travel and dubbed it `SpecLockMap`<br><sup>&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;&nbsp;or: "how we accidentally unlocked `rr` on AMD Zen"</sup>

Having not made as much progress in removing noise by subtracting interrupts, as we were hoping, focus shifted to building the tools necessary to delve deeper into the `-Z self-profile` data, to narrow down which parts of `rustc` were being affected.

What we came up is `summarize aggregate` (`summarize` being the main tool for inspecting `measureme` profiles). [The pull request](https://github.com/rust-lang/measureme/pull/129) has more details and examples, but in short, it:
1. takes a group of profiles collected from identical `rustc` executions (we were using groups of 10 runs)
2. lines up every event endpoint (i.e. starts and ends) across the entire group, making sure the stream of events has identical details (other than the timestamps/instruction counts)
3. compares the distance (in time/instruction count) between every consecutive pair of event endpoints, and shows the best and worst such intervals (in terms of variation between runs)

From the instructions-counting example in the PR, there were the 2 smallest and largest variances, at the time:
```
  ±0 instructions: 1701795 occurrences, or 99.61%
  ±0.5 instructions: 519 occurrences, or 0.03%
```
```
  ±659 instructions: `free_global_ctxt()`
  ±1147 instructions: in `expand_crate()`, after `pre_AST_expansion_lint_checks()`
```

The important aspect here is that every interval corresponds to a consecutive pair of time/counter samples, so even if e.g. reading the counter was imprecise by a few instructions, it could not add up, as it might when looking at `summarize summarize` output.

Additionally, we had two avenues for learning more about what was executing in those high-noise intervals:
* adding more fine-grained events to `rustc`, e.g. for only a few statements in a function
* enabling the serialization of function arguments, [via `-Zself-profile-events=default,args`](https://doc.rust-lang.org/nightly/unstable-book/compiler-flags/self-profile-events.html), for the events that support it

The main outcome of employing these techniques was learning that the noise is quite rare, and highly correlated with heap allocation/deallocation (i.e. `malloc`/`realloc`/`free`) of `rustc`'s AST/IRs (macro expansion on the AST, AST->HIR lowering, MIR transformations).

We couldn't find any obvious sources of non-determinism in `glibc`'s heap allocator, and `strace` wasn't showing a difference in the order, arguments, or return values of syscalls (other than process/thread IDs), so we decided to try and replace the allocator used by `rustc` with a simpler one written in Rust (such as [`wee_alloc`](https://crates.io/crates/wee_alloc), commonly used for WASM).

As `#[global_allocator]` wasn't compatible with `rustc`'s reliance on `dylib` crates (used to avoid duplication on disk between `rustc`, `rustdoc`, `clippy`, `miri`, etc.), we had to write [an adapter between C heap functions and a Rust allocator](https://gist.github.com/eddyb/df25e28313b37c8c9519bc503541f4b0), but once we had that we could start comparing simpler allocators to `glibc`'s, in terms of noise.

Surprisingly, both `wee_alloc` and simpler allocators like `bump_alloc` (which never deallocates any memory), had similar noise to `glibc`, though deallocation became deterministic with `bump_alloc` (as `free` was a noop).

So we turned our attention to synthetic benchmarking again, and [added `bump_alloc` allocation to `rdpmc-bench.rs`](https://gist.github.com/eddyb/269491b0b59605f39f1d0a9dcf535c4a/8273ea67921c127d89d7aacf2da82dec41a23bb1), which produced occasional overcounting (even if it could take a while for it to happen).

One important detail here is that the heap allocator noise we were seeing was on the AMD Zen 1 EPYC server (we were relying on it as it was otherwise the least noisy machine we had access to), and we could only reproduce the occasional overcounting (in the synthetic benchmark) on AMD Zen CPUs (including Zen 2).

At this point our assumption became that this was a Zen-specific bug, and we quickly reduced the `bump_alloc` allocation function down to a single atomic `fetch_add` (i.e. the `lock xadd` instruction).

The `lock xadd` overcounting suspiciously came in *exact multiples* of the number of instructions in the loop body, *as if* the benchmark loop was doing extra iterations - e.g. instead of `5000` instructions for `1000` iterations, we might see `5005` or `5010`, when the loop body was `5` instructions - and this effect remained as we added extra `nop`s to the loop body.

This was our first indication that something much stranger than a mere faulty counter, was happening here: it *shouldn't* be possible for the hardware to redo *whole* loop iterations, especially with a conditional branch (the loop backedge) in the overcounted instructions.

But [**@m-ou-se**](https://github.com/m-ou-se) said the unthinkable out loud *anyway*:
> is it speculatively executing until the next `lock` which then conflicts with the existing `lock` and throws the whole thing out?

After a bit more experimentation to confirm that the distance between consecutive `lock xadd` instructions mattered, rather than the loop itself, we started looking for any mentions of such a behavior, and [the Google search query `instructions retired lock`](https://www.google.com/search?q=instructions+retired+lock) had (at the time) this as one of the results: https://www.freepatentsonline.com/y2018/0074977.html

That is AMD's **US20180074977A1** ([read on Google Patents](https://patents.google.com/patent/US20180074977A1/en)), which has this in its abstract:
> A lock instruction and younger instructions are allowed to speculatively retire prior to the store portion of the lock instruction committing its value to memory. (...) In the event that the processor detects a violation (...), the processor rolls back state and executes the lock instruction in a slow mode in which younger instructions are not allowed to retire until the stored value of the lock instruction is committed.

(Here "younger instructions" refers to instructions *after* the atomic, i.e. `lock`, instruction, which would always execute *later* in a simple "in-order" CPU, but modern "out-of-order" CPUs do some amount of on-the-fly reordering of instructions, hence the specific terminology)

This was the confirmation we needed: AMD decided to "transactionally" execute (or rather, speculate) instructions past an atomic, without any special opt-in from software, and while the CPU *was* able to "time-travel" back to the atomic instruction, it wasn't saving performance counters in order to restore them as well.

What could we *even* do about missing silicon like this?
Well, it so happens that we live in a post-[Spectre](https://en.wikipedia.org/wiki/Spectre_(security_vulnerability)) world, and while the mitigations for it (and related speculative execution vulnerabilities) have had a negative impact on performance, there is an upside: both Intel and AMD have revealed previously undocumented special registers, and bits in them which can turn off various forms of speculation.

Having that ability all along (for anything non-essential in a CPU) makes sense given silicon development cycles, and it would be *especially* beneficial for AMD to have such a disable bit for their new "speculate past atomics, possibly having to roll all that back later" optimization, *just in case* there are flaws found later (at which point an UEFI "BIOS" update would set the bit).

So we were expecting there would be a disable bit and that we might be able to find it, especially given our synthetic benchmark, though randomly changing undocumented special registers (aka MSRs, or "model-specific registers") can be somewhat dangerous, mostly in terms of corrupting the OS and/or running processes (but it shouldn't be easy/possible to permanently damage the hardware itself).

Regarding the synthetic benchmark, [**@puckipedia**](https://github.com/puckipedia) came up with the idea of using a child thread to constantly interfere with the same cache line the main thread was `lock xadd`-ing to, and this led to *constant* overcounting, instead of merely occasional, which made it much easier to confirm whether an undocumented bit had *any* relevant effect.

Before we were able to ever try a single MSR bit, however, [**@jix**](https://github.com/jix) already tried a few and found that `MSRC001_102C[55]` (i.e. bit `55` of MSR `0xC001_102C`) solved the overcounting problem, when set. That was already great, but then they found that the [ASRock "B450M Pro4" motherboard manual](https://download.asrock.com/Manual/B450M%20Pro4.pdf) contains this description for "Enable IBS":
> Enables IBS through `MSRC001_1005[42]` and disables `SpecLockMap` through `MSRC001_1020[54]`.

Not only did `MSRC001_1020[54]` *also* solve our overcounting problem, we now had a name for this "post-`lock` speculation" feature: **`SpecLockMap`**. More recently, [someone from AMD implied](https://github.com/jlgreathouse/AMD_IBS_Toolkit/issues/5#issuecomment-702828173) that ASRock effectively leaked the correct name, so we'll keep using it.
While not officially documented by AMD, the fact that "UEFI Setup" (aka "BIOS") toggles may set this bit, meant it was less likely to do anything *else*, and we decided to stick with it (rather than the other, completely unknown, bit, or any others).

By this point, we had heard that the [`rr` ("Record and Replay")](https://rr-project.org/) tool didn't work on AMD Zen CPUs because of some accuracy issue with hardware performance counters, so [we left a comment](https://github.com/mozilla/rr/issues/2034#issuecomment-691339758) about `SpecLockMap` and how to disable it, in case that might help them. [**@glandium**](https://github.com/glandium) quickly confirmed disabling `SpecLockMap` did the trick, and started looking into remaining unrelated issues blocking `rr` on AMD Zen CPUs (now that the performance counters could be made reliable).

While almost everyone testing our synthetic benchmark or `rr`, was using an AMD Zen 2 (Ryzen `3xxx`) CPU, and could use tools like `rdmsr`/`wrmsr` to test MSR bits, our EPYC server was (and remains) Zen 1, and so we ran into another problem because of that: [the mitigation for "Speculative Store Bypass"](https://developer.amd.com/wp-content/resources/124441_AMD64_SpeculativeStoreBypassDisable_Whitepaper_final.pdf) (i.e. its disable bit) is `MSRC001_1020[10]` on Zen CPUs predating Zen 2, and the Linux kernel caches the value of the MSR in order to set/unset that one bit, meaning any *other* changes (such as our bit `54` to disable `SpecLockMap`) to the same MSR would get periodically reverted.

To work around this problem, [we developed](https://github.com/mozilla/rr/issues/2034#issuecomment-691622904) a small [kernel module](https://gist.github.com/eddyb/b888bb87988ca97ead9abcf96aa49e15) to override the cached MSR value, which [**@glandium**](https://github.com/glandium) later replaced with [an improved version](https://github.com/mozilla/rr/wiki/Zen#kernel-module) that instead hooked *any* writes to `MSRC001_1020`, via the kernel's own "tracepoints" system, to also set bit `54`.

Given the `SpecLockMap` name, [we later stumbled over](https://github.com/mozilla/rr/issues/2034#issuecomment-693279404) a once-documented (only in an older Zen 1 EPYC manual) performance counter named `SpecLockMapCommit` (`r0825:u`), and [**@jix**](https://github.com/jix) came up with the idea of using it to [detect whether `SpecLockMap` was active](https://github.com/mozilla/rr/issues/2034#issuecomment-694127840) (and warn the user that they should try to disable it). All it took was one (uncontended) atomic instruction for `SpecLockMapCommit` to increment - presumably made more reliable by us pairing `rdpmc` (used both before and after the atomic) with a serializing instruction.

Digging through various manuals, [we found several ways](https://github.com/mozilla/rr/issues/2034#issuecomment-695780661) in which the `lock` performance counters were documented, but the main takeaway is that `SpecLockMapCommit` was renamed to `SpecLockHiSpec` for Zen 2, while serving the same function.

When all was said and done:
* `rr` was now working on AMD CPUs (and we'd heard multiple people claim that the only reason they didn't go with an AMD CPU was the lack of `rr` support)
* the variance in `instructions-minus-irqs:u` totals went away (except non-determinism sources we could quantify, e.g. creating a temp dir with a random name)
* both `rr` and our implementation could detect `SpecLockMap` and link the user to https://github.com/mozilla/rr/wiki/Zen for more information on how to disable it

### `jemalloc`: purging will commence in ten seconds

Up until this point, we've focused on testing with the default `glibc` allocator, but we had seen an ASLR-like effect when using `jemalloc`, that we couldn't track down originally. But official builds use `jemalloc`, so we had to take another look.

With the `SpecLockMap` noise gone from non-`jemalloc` results, we were now sure some kind of non-determinism was being introduced by `jemalloc`, and `strace` confirmed some memory ranges are released back to the OS at different times, resulting in unpredictable addresses for later memory allocations (and therefore act like a weak form of ASLR).

By looking at all the places in `jemalloc` which can call `madvise(..., MADV_FREE)` (the initial non-deterministic syscall), we came across [a time-delayed "purging" mechanism](http://jemalloc.net/jemalloc.3.html#opt.dirty_decay_ms), which perfectly explained what we were seeing: when *exactly* (in the execution of the program) the timer would fire, is unpredictable, and the ordering of the `madvise(..., MADV_FREE)` syscall respective to all the `mmap` syscalls, would determine all later memory addresses.

However, this "purging" would only happen every 10 seconds by default, so anything taking less time to compile should be just as deterministic instruction-wise, with `jemalloc` as with `glibc`'s allocator. It just so happened that our testcase would take around 13 seconds

We were able to disable the time-delayed mechanism by either setting environment variable (`MALLOC_CONF=dirty_decay_ms:0,muzzy_decay_ms:0`) or by placing that same value in a `static` with the symbol name `malloc_conf`, inside `rustc`.

Some experiments were attempted using Rust PR [#77162](https://github.com/rust-lang/rust/pull/77162), but as most `perf.rust-lang.org` benchmarks take less than 10 seconds, `jemalloc` wouldn't behave differently for them.

Sadly, losing this "purging" feature might come at some performance loss, but there may be a way to disable it as soon as `rustc` knows it's being ran with `-Z self-profile`, if necessary.

### Rebasing *shouldn't* affect the results, right?

In order to be able to submit the `measureme` and `rustc` PRs, we had to first update our changes to latest Rust (i.e. `git rebase` onto latest `rust-lang/rust`), and after doing so we got another set of measurements, just to make sure everything still works.

To our disappointment, there was now noise that wasn't there before the rebase, and we decided to track it down, just in case we made a significantly flawed assumption along the way.
Using binary search on the PR merge commits (as listed by `git log --first-parent`) between our starting point, and the latest `rust-lang/rust`, we found 3 relevant PRs:
1. [#75600](https://github.com/rust-lang/rust/pull/75600) "Improve codegen for `align_offset`" by [**@nagisa**](https://github.com/nagisa): ~±2 instructions
    * seems to have went away since
2. [#75642](https://github.com/rust-lang/rust/pull/75642) "Move doc comment parsing to rustc_lexer" by [**@matklad**](https://github.com/matklad): ~±100 instructions
    * specifically [this commit](https://github.com/rust-lang/rust/pull/75642/commits/ccbe94bf77e6a32fc9f31425bc820345be3143c0) turned out to be responsible
    * we were able to work around it by using `match_indices` to further simplify `cook_doc_comment` (removing the `str::contains` call in the process)
3. [#73905](https://github.com/rust-lang/rust/pull/73905) "Separate projection bounds and predicates" by [**@matthewjasper**](https://github.com/matthewjasper): ~±100 instructions

We haven't been able to identify an explanation as to what is actually happening in *any* of these cases, so for now we've worked around them, but number 3 was much harder.

By introducing more fine-grained `measureme` intervals, and repeatedly splitting them into smaller and smaller intervals, we were able to shrink the source of the noise to [one loop](https://github.com/rust-lang/rust/blob/08e2d4616613716362b4b49980ff303f2b9ae654/compiler/rustc_trait_selection/src/traits/select/candidate_assembly.rs#L174-L197).

However, any attempt at measuring `candidate_should_be_dropped_in_favor_of` itself resulted in the noise going away. While this stopped us from narrowing down the source of the noise further, it did suggest the serializing instruction (which we use for `rdpmc`) was helping.

And indeed, `cpuid` (i.e. what we use as the serializing instruction for `rdpmc`) alone is enough to stop the noise, and so is `mfence`. We kept the latter because it's simpler, though it's not clear which one would be cheaper, or what impact on performance it even has.

Our theory right now is that LLVM (rarely) generates an instruction which has a performance counting bug (unless serialized), but further work is necessary to confirm or disprove that.

### Epilogue: Zen's undocumented 420 counter

In the process of investigating the post-rebase issues above, we were reminded of page faults, and how, while we never observed `perf stat -e page-faults:u ...` produce different totals across runs, they still overcount the instruction counter the same way IRQs do, so non-deterministic page faults would result in instructions-counting noise.

But unlike with IRQs, there is no documented hardware performance counter for "page faults", or "CPU exceptions" in general, so there is no easy way to undo the associated overcounting.
(`page-faults:u` is a "software event", i.e. the kernel manually tracks it, so while we used it in our investigations, it's no substitute for a fault/exception counter implemented in hardware)

However, at least for AMD Zen CPUs, there are obvious small gaps in the numbering of the documented counters, so we decided to try and find an undocumented counter of the right magnitude (that of `page-faults:u`), hiding in those gaps.

Most performance counters are configured with a 16-bit `0xMM_XX` (`rMMXX:u` in `perf` notation) value, where `XX` and `MM` are:
* `XX` aka "event selector": a numeric index (sequential except for undocumented gaps) corresponding to one event, or a group of related events
* `MM` aka "mask": a bitset of which events, sharing the same event selector, are enabled, or entirely ignored for simpler events

Because of how forgiving the hardware is, one can just iterate through event selectors, and always set the mask to `0xff` to get the maximal number of events counted, no matter how many bits are *actually* supported for that event selector.
The main limitation is that there are only 6 counters that can be active at any time, so the test workload has to be run for batches of 6, e.g. for `0x06..=0x0b` event selectors:
```shell
perf stat -e rff06:u,rff07:u,rff08:u,rff09:u,rff0a:u,rff0b:u ...
```

Once an event selector is known to be produce any (non-zero) results, or if it specifically results in an interesting value, each bit of the mask can be checked separately, i.e. `r01XX:u,r02XX:u,r04XX:u,r08XX:u,...`, in order to determine which bits are functional.

We went through a few dozen counters before stumbling over one (`rff20:u`) which was much closer in magnitude to `page-faults:u` than any others, and only one of the mask bits seemed active, resulting in `r0420:u` as the simplest way to refer to the counter.

Looking at `page-faults:u`, IRQs (`r002c:u`) and `r0420:u` together, it became clear that `r0420:u` was only barely larger than the sum of `page-faults:u` and IRQs, leading us to speculate that it's either counting *both* hardware interrupts and CPU exceptions, or perhaps even the `iret`s the kernel uses to return from handling any of those interrupts/exceptions.

We might be able to figure out what it actually is by using the 32-bit x86 syscall interface (`int 0x80`) to `read` the `perf_event_open` pseudo-file, which would cause the kernel to `rdpmc` between the interrupt firing and the `iret` back to userspace, and by comparing the read value with what userspace `rdpmc` reports, just after the syscall returns.
However, we have not attempted this yet, and so it remains as possible future work (if we want to start using this counter).

The main usefulness of a general counter like this (as opposed to just IRQs) is being able to account for non-deterministic page faults, which may be more of a problem on machines where context-switching, or even swapping parts of RAM to disk, are more common.
