#!/bin/bash

CLIENT_BIN="socat"
#CLIENT_BIN="./tokio_client/target/release/tokio_client"

for i in {0..19}; do
  pv -c ramdisk/rand.dat | "${CLIENT_BIN}" - 'TCP:127.0.0.1:4242' > /dev/null &
done
wait
