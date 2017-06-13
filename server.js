#!/usr/bin/env node
const net = require("net");

const size = () => Object.getOwnPropertyNames(clients).length;
const clients = {};

net.createServer(c => {
  const addr = `${c.remoteAddress}:${c.remotePort}`;
  c.setMaxListeners(Infinity);
  const onStop = () => {
    delete clients[addr];
    console.log(`Client disconnected, ${size()} clients`);
  };

  c.on('end', onStop);
  c.on("error", onStop);
  for (const k in clients) {
    c.pipe(clients[k], { end: false }).pipe(c, { end: false });
  }
  c.pipe(c);
  clients[addr] = c;
  console.log(`Client connected, ${size()} clients`);
}).listen(4242);
