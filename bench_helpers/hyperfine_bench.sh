#!/bin/bash -eu
out=$(mktemp)
hyperfine --export-json=$out "$@" &>/dev/null
jq -c '{wall_time: .results[0].mean, user: .results[0].user, system: .results[0].system }' <$out
