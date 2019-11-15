#!/bin/bash -eu
benchprog=$1; shift;
base=$(git merge-base HEAD^ @{u})
head=$(git rev-parse HEAD)
cbdr sample $benchprog $base $head | cbdr diff $@ $base,$head
