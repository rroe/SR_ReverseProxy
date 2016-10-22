#![feature(phase)]
#[phase(plugin, link)] extern crate log;
extern crate green;
extern crate rustuv;

use std::io;
use std::os;
use std::io::{Listener,Acceptor,TcpStream};

#[start]
fn start(argc: int, argv: *const *const u8) -> int {
    green::start(argc, argv, rustuv::event_loop, main)
}


fn main() {
    let args = os::args();
    if args.len() < 3 {
        println!("Usage: {} <ip> <port>", args[0]);
        os::set_exit_status(1);
        return
    }
    let host = args[1].as_slice();
    let port = from_str::<u16>(args[2].as_slice()).unwrap();

    let sock = io::TcpListener::bind(host, port).unwrap();
    let mut acceptor = sock.listen();
    debug!("Listening...");
    for stream in acceptor.incoming() {
        match stream {
            Err(e) => warn!("Accept error: {}", e),
            Ok(stream) => {
                spawn(proc() {
                    println!("{}", handle_client(stream));
                })
            }
        }
    }
}

type Buf = [u8, ..10240];

fn handle_client(mut stream: io::TcpStream) -> io::IoResult<()> {
    info!("New client {}", stream.peer_name());
    let mut buf: Buf = [0u8, ..10240];
    let (child_tx, parent_rx) = channel::<Buf>();
    let (parent_tx, child_rx) = channel::<Buf>();

    spawn(proc() {
        // if this `deschedule` will be commented, only one CPU core will be used (rust 0.11)
        std::task::deschedule();
        for mut buf in child_rx.iter() {
            for _ in range::<u8>(0, 20) {
                buf.reverse();
            }
            child_tx.send(buf);
        };
    });
    loop {
        let got = try!(stream.read(buf));
        if got == 0 {
            // Is it possible? Or IoError will be raised anyway?
            break
        }
        // outsource CPU-heavy work to separate task, because current green+libuv
        // implementation bind all IO tasks to one scheduler (rust 0.11)
        // see https://botbot.me/mozilla/rust/2014-08-01/?msg=18995736&page=11
        parent_tx.send(buf);
        let to_send: Buf = parent_rx.recv();
        try!(stream.write(to_send.slice(0, got)));
    }
    Ok(())
}
