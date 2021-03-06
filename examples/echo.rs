//! An echo server that just writes back everything that's written to it.
//!
//! If you're on unix you can test this out by in one terminal executing:
//!
//! ```sh
//! $ cargo run --example echo
//! ```
//!
//! and in another terminal you can run:
//!
//! ```sh
//! $ nc localhost 8080
//! ```
//!
//! Each line you type in to the `nc` terminal should be echo'd back to you!

extern crate env_logger;
extern crate futures;
extern crate tokio_core;
#[macro_use]
extern crate tokio_fiber;

use std::env;
use std::net::SocketAddr;

use futures::Future;
use futures::stream::Stream;
use tokio_core::net::TcpListener;
use tokio_core::reactor::Core;

use std::io::{self, Read, Write};

fn main() {
    env_logger::init().unwrap();
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:8080".to_string());
    let addr = addr.parse::<SocketAddr>().unwrap();

    // Create the event loop that will drive this server
    let mut l = Core::new().unwrap();
    let handle = l.handle();

    // Create a TCP listener which will listen for incoming connections
    let socket = TcpListener::bind(&addr, &handle).unwrap();

    // Once we've got the TCP listener, inform that we have it
    println!("Listening on: {}", addr);

    // Pull out the stream of incoming connections and then for each new
    // one spin up a new task copying data.
    //
    // We use the `io::copy` future to copy all data from the
    // reading half onto the writing half.
    let done = socket.incoming().for_each(move |(mut conn, _addr)| {
        let fib = tokio_fiber::Fiber::new(move || -> io::Result<()> {
            let mut buf = [0u8; 1024 * 64];
            loop {
                let size = poll!(conn.read(&mut buf))?;
                if size == 0 {/* eof */ break; }
                let _ = poll!(conn.write_all(&mut buf[0..size]))?;
            }

            Ok(())
        });

        let fib = fib.map_err(|e| {
            println!("error: {}", e);
        });

        handle.spawn(fib);

        Ok(())
    });
    l.run(done).unwrap();
}
