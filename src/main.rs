extern crate time;

use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::Read;
use std::io::Write;

fn timestamp () -> f64 {
    let timespec = time::get_time();
    // 1459440009.113178
    let mills: f64 = timespec.sec as f64 + (timespec.nsec as f64 / 1000.0 / 1000.0 / 1000.0 );
    mills
}

fn handle_client(mut stream: TcpStream) {
    let mut buf;

	let start_time = timestamp();

    loop {
        // clear out the buffer so we don't send garbage
        buf = [0; 1024];
        let _ = match stream.read(&mut buf) {
            Err(e) => panic!("Got an error: {}", e),
            Ok(m) => {
				let elapsed_time = timestamp() - start_time;
				println!("Current elapsed time: {}", elapsed_time);
                if m == 0 {
                    // we've got an EOF
                    break;
                }
                m
            },
        };

		let mut s = String::new();
		for num in buf.iter() {
			let num: u8 = *num;
			s.push(num as char);
		}
		println!("===============[START]===============\n{}\n================[END]================\n", s);

		let result: String = String::from("<html><head><title>Test!</title></head><body>Hello from Rust!</body></html>");

        match stream.write(result.as_bytes()) {
            Err(_) => break,
            Ok(_) => continue,
        }
    }
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:88").unwrap();
	println!("Starting TCP listener...");
    for stream in listener.incoming() {
    	match stream {
            Err(e) => { println!("failed: {}", e) }
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream)
                });
            }
        }
    }
}
