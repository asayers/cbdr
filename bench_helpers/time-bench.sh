#!/bin/bash -eu

# wall_time: seconds
# user_time: seconds
# system_time: seconds
# max_rss: KB
/usr/bin/time -f'{ "wall_time": %e, "user_time": %U, "system_time": %S, "max_rss": %M }' $@ 2>&1 >/dev/null
