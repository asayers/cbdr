#!/bin/bash -eu

# We're using `time` to measure the max RSS, but note that it includes the
# RSS of the `perf` program too.
/usr/bin/time -f'%M,KB,max-rss,,,,' -- \
    perf stat --field-separator="," -- \
    $@ 2>&1 >&- |
    awk -F, '
BEGIN { print "task clock,units,cpu utilization,context switches,cpu migrations,page faults,cycles,instructions,branches,branch misses,max rss" }
/,task-clock,/ { task_clock = $1; clock_units = $2; cpu_utilization = $6 }
/,context-switches,/ { context_switches = $1 }
/,cpu-migrations,/ { cpu_migrations = $1 }
/,page-faults,/ { page_faults = $1 }
/,cycles,/ { cycles = $1 }
/,instructions,/ { instructions = $1 }
/,branches,/ { branches = $1 }
/,branch-misses,/ { branch_misses = $1 }
/,max-rss,/ { max_rss = $1 }
END { print task_clock","units","cpu_utilization","context_switches","cpu_migrations","page_faults","cycles","instructions","branches","branch_misses","max_rss }
'
