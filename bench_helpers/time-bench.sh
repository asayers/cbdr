#!/bin/bash -eu

# wall_time: seconds
# user_time: seconds
# system_time: seconds
# max_rss: KB
out=$(mktemp)
/usr/bin/time -o$out -f'{ "wall_time": %e, "user_time": %U, "system_time": %S, "max_rss": %M }' $@ &>/dev/null
cat $out
