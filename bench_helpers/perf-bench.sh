#!/bin/bash -eu

# We're using `time` to measure the max RSS, but note that it includes the
# RSS of the `perf` program too.
perf stat --field-separator="," -- \
    $@ 2>&1 >&- |
    awk -F, '
BEGIN { print "{" }
/,task-clock,/ {
    print "\t\"task_clock\": " $1 ",";
    print "\t\"cpu_utilization\": " $6 ",";
}
/,context-switches,/ { print "\t\"context_switches\": " $1 "," }
/,cpu-migrations,/ { print "\t\"cpu_migrations\": " $1 "," }
/,page-faults,/ { print "\t\"page_faults\": " $1 "," }
/,cycles,/ { print "\t\"cycles\": " $1 "," }
/,instructions,/ { print "\t\"instructions\": " $1 "," }
/,branches,/ { print "\t\"branches\": " $1 "," }
/,branch-misses,/ { print "\t\"branch_misses\": " $1 }
END { print "}" }
'
