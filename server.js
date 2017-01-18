#!/usr/bin/env node
const net = require("net");
const _ = require("lodash");

const clients = {};

net.createServer(c => {
  const addr = `${c.remoteAddress}:${c.remotePort}`;
  c.setMaxListeners(Infinity);
  const onStop = () => {
    delete clients[addr];
    console.log(`Client disconnected, ${_.size(clients)} clients`);
  };

  c.on('end', onStop);
  c.on("error", onStop);
  _.forOwn(clients, (o) => {
    c.pipe(o, { end: false }).pipe(c, { end: false });
  });
  c.pipe(c);
  clients[addr] = c;
  console.log(`Client connected, ${_.size(clients)} clients`);
}).listen(4242);
