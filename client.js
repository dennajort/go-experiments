#!/usr/bin/env node
const net = require("net");
const sock = net.createConnection(4242, "localhost", () => {
  process.stdin.pipe(sock).pipe(process.stdout);
});

sock.on("end", () => { process.exit(0) });
