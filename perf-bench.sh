#!/bin/bash -eu

RESULTS=$(mktemp)
perf stat --field-separator="," --output=$RESULTS -- $@ >/dev/null

TASK_CLOCK=$(     grep ',task-clock,'    <$RESULTS | xsv select 1)
CLOCK_UNITS=$(    grep ',task-clock,'    <$RESULTS | xsv select 2)
CPU_UTILIZATION=$(grep ',task-clock,'    <$RESULTS | xsv select 6)
INSTRUCTIONS=$(   grep ',instructions,'  <$RESULTS | xsv select 1)
CYCLES=$(         grep ',cycles,'        <$RESULTS | xsv select 1)
BRANCHES=$(       grep ',branches,'      <$RESULTS | xsv select 1)
BRANCH_MISSES=$(  grep ',branch-misses,' <$RESULTS | xsv select 1)

echo "commit,timestamp,task clock,units,cpu utilization,instructions,cycles,branches,branch misses"
echo "$COMMIT,$(date -Iseconds),$TASK_CLOCK,$CLOCK_UNITS,$CPU_UTILIZATION,$INSTRUCTIONS,$CYCLES,$BRANCHES,$BRANCH_MISSES"
