extern crate time;

use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::Read;
use std::io::Write;

fn timestamp() -> f64 {
    let timespec = time::get_time();
    let seconds: f64 = timespec.sec as f64 + (timespec.nsec as f64 / 1000.0 / 1000.0 / 1000.0 );
    seconds
}

fn get_string_from_buffer_string(string: String) -> String {
	let mut out = String::new();
	for c in string.chars() {
		if (c as u8) != 0 {
			out.push(c);
		} else {
			break;
		}
	}
	out
}

fn end_in_two_nl(string: &String) -> bool {
	if string.len() < 2 {
		return false;
	}
	let mut last_was_nl = false;
	for c in string.chars() {
		if (c as u8) == 13 || (c as u8) == 0 {
			continue;
		}
		if (c as u8) == 10 {
			if last_was_nl == true {
				return true;
			} else {
				last_was_nl = true;
			}
		} else {
			last_was_nl = false;
		}
	}
	false
}

fn length_u8_array(buffer: &[u8]) -> i32 {
	let mut out: i32 = 0;
	for b in buffer {
		if *b != 0 {
			out = out + 1;
		} else {
			return out;
		}
	}
	out
}

fn handle_client(mut stream: TcpStream) {
	println!("New connection established!");
	let start_time = timestamp();

	let result: String = String::from("<html><head><title>Test!</title></head><body>Hello from Rust!</body></html>");

	let mut from_client = String::new();

	'outer: loop {
	    'inner: loop {
	        let mut buf_tmp = [0; 1024];
	        let _ = match stream.read(&mut buf_tmp) {
	            Err(e) => panic!("Got an error: {}", e),
	            Ok(m) => {
					if m == 0 {
						// Break on EOF
						println!("Reached EOF!");
						break 'inner;
					}
					let mut s = String::new();
					for num in buf_tmp.iter() {
					    let num: u8 = *num;
					    s.push(num as char);
					}
					let buf_string = get_string_from_buffer_string(s);
					from_client.push_str(&buf_string);
					if end_in_two_nl(&from_client) == true {
						break 'inner;
					}
	                m
	            },
	        };
	    }
		let elapsed_time = timestamp() - start_time;
		println!("Elapsed time: {} seconds", elapsed_time);
		match stream.write(result.as_bytes()) {
			Err(_) => {
				break 'outer;
			}
			Ok(_) => {
				break 'outer;
				println!("Total: {}", from_client);
				return (); // Force close socket/thread
			}
		}
	}
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:88").unwrap();
	println!("Starting TCP listener...");
    for stream in listener.incoming() {
    	match stream {
            Err(e) => { println!("Failed: {}", e) }
            Ok(stream) => {
                thread::spawn(move || {
                    handle_client(stream)
                });
            }
        }
    }
}
