#!/bin/bash

for i in {0..19}; do
  pv -c ramdisk/rand.dat | socat - 'TCP:127.0.0.1:4242' > /dev/null &
done
wait
