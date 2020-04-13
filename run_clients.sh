#!/bin/bash

for i in {0..29}; do
  pv -c rand.dat | socat - 'TCP:127.0.0.1:4242' > /dev/null &
done

for job in `jobs -p`
do
echo $job
    wait $job
done
