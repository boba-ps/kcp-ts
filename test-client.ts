//
// Copyright (c) 2022 chiya.dev
//
// Use of this source code is governed by the MIT License
// which can be found in the LICENSE file and at:
//
//   https://opensource.org/licenses/MIT
//
import { createSocket } from "dgram";
import { Kcp } from "./kcp";

const start = performance.now();

const server = createSocket("udp4");
const kcp = new Kcp(69, 420, (buffer) => {
  if (!buffer.length) {
    console.error("zero buffer!!");
    return;
  }

  // console.debug(kcp);
  server.send(buffer, 6800, "127.0.0.1");
  // console.debug(
  //   `wrote ${buffer.length} bytes to udp socket: ${buffer.toString("hex")}`
  // );
});

server.on("error", (err) => {
  console.error(err);
});

const recvBuffer = Buffer.alloc(0x20000);

server.on("message", (buffer) => {
  console.error(
    `read ${buffer.length} bytes from udp socket: ${buffer.toString("hex")}`
  );

  const result = kcp.input(buffer);
  if (result < 0) {
    console.error(`bad result on recv: ${result}`);
    return;
  } else {
    // console.debug(`input ${result} bytes into kcp obj`);
  }

  kcp.update(performance.now() - start);
  kcp.flush();

  for (;;) {
    const read = kcp.recv(recvBuffer);
    if (read < 0) break;
    else process.stdout.write(recvBuffer.slice(0, read));
  }
});

process.stdin.on("data", (buffer) => {
  const result = kcp.send(buffer);
  if (result < 0) {
    console.error(`bad result on send: ${result}`);
    return;
  }

  kcp.update(performance.now() - start);
  kcp.flush();
});

server.bind(6801, "127.0.0.1");
setInterval(() => kcp.update(performance.now() - start), 100);
