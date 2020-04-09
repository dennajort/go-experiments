#!/bin/bash

for i in {0..9}; do
  pv -c rand.dat | ./client_rust > /dev/null &
done

for job in `jobs -p`
do
echo $job
    wait $job
done
