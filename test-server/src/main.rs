//
// Copyright (c) 2022 chiya.dev
//
// Use of this source code is governed by the MIT License
// which can be found in the LICENSE file and at:
//
//   https://opensource.org/licenses/MIT
//
#[macro_use]
extern crate log;

use kcp::Kcp;
use std::{
    env,
    io::{stdin, stdout, Read, Write},
    net::UdpSocket,
    sync::{Arc, Mutex},
    thread::{sleep, spawn},
    time::{Duration, Instant},
};

use crate::error::Error;

mod error;
mod kcp;

struct UdpWriter(Arc<UdpSocket>);

impl Write for UdpWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let written = self
            .0
            .send_to(buf, env::var("KCP_DST").unwrap_or("127.0.0.1:6801".into()))?;

        debug!("wrote {written} bytes to udp socket: {}", hex::encode(buf));
        Ok(written)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

fn main() {
    env_logger::init();

    let start = Instant::now();
    let socket = Arc::new(
        UdpSocket::bind(env::var("KCP_SRC").unwrap_or("127.0.0.1:6800".into()))
            .expect("failed to bind udp socket"),
    );
    let kcp = Arc::new(Mutex::new(Kcp::new(69, 420, UdpWriter(socket.clone()))));

    let reader = spawn({
        let socket = socket.clone();
        let kcp = kcp.clone();
        move || read(start, socket, kcp)
    });

    let writer = spawn({
        let socket = socket.clone();
        let kcp = kcp.clone();
        move || write(start, socket, kcp)
    });

    let updater = spawn({
        let kcp = kcp.clone();
        move || loop {
            kcp.lock()
                .unwrap()
                .update(start.elapsed().as_millis() as u32)
                .expect("failed to update kcp instance");

            sleep(Duration::from_millis(100));
        }
    });

    reader.join().unwrap();
    writer.join().unwrap();
    updater.join().unwrap();
}

fn read(start: Instant, socket: Arc<UdpSocket>, kcp: Arc<Mutex<Kcp<UdpWriter>>>) {
    let mut output = stdout().lock();
    let mut buffer = vec![0; 0x20000];

    loop {
        let read = socket
            .recv(&mut buffer)
            .expect("failed to read from udp socket");

        debug!(
            "read {read} bytes from udp socket: {}",
            hex::encode(&buffer[..read])
        );

        let mut kcp = kcp.lock().unwrap();

        kcp.input(&buffer[0..read])
            .expect("invalid kcp packet received");

        kcp.update(start.elapsed().as_millis() as u32)
            .expect("failed to update kcp instance");

        kcp.flush().expect("failed to flush kcp stream");

        loop {
            let read = match kcp.recv(&mut buffer) {
                Ok(read) => read,
                Err(Error::RecvQueueEmpty) => break,
                Err(Error::ExpectingFragment) => break,
                err @ Err(_) => err.expect("failed to read from kcp stream"),
            };

            debug!("read {read} bytes from kcp stream");

            output
                .write_all(&buffer[..read])
                .expect("failed to write to stdout");
        }
    }
}

fn write(start: Instant, _socket: Arc<UdpSocket>, kcp: Arc<Mutex<Kcp<UdpWriter>>>) {
    let mut input = stdin().lock();
    let mut buffer = vec![0; 0x20000];

    loop {
        let read = input.read(&mut buffer).expect("failed to read from stdin");
        if read == 0 {
            break;
        }

        let mut total = 0;

        while total != read {
            let mut kcp = kcp.lock().unwrap();

            let written = kcp
                .send(&buffer[total..read])
                .expect("failed to write to kcp stream");

            debug!("wrote {written} bytes to kcp stream");

            kcp.update(start.elapsed().as_millis() as u32)
                .expect("failed to update kcp instance");

            kcp.flush().expect("failed to flush kcp stream");

            total += written;
        }
    }
}
