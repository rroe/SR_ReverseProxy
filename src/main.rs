extern crate time;

use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::Read;
use std::time::Duration;
use std::io::Write;

fn timestamp() -> f64 {
    let timespec = time::get_time();
    let seconds: f64 = timespec.sec as f64 + (timespec.nsec as f64 / 1000.0 / 1000.0 / 1000.0 );
    seconds
}

fn proxy_req_to_localhost(client_req: String) -> String{
    let result_err: String = String::from("<html><head><title>Reverse Proxy</title></head><body>Unknown Error Occurred.</body></html>");
    let mut socket = TcpStream::connect("127.0.0.1:80").unwrap();
    let _ = match socket.write(client_req.as_bytes()) {
        Err(e) => {
            println!("[ERROR] on proxy: {}", e);
            return result_err;
        },
        Ok(m) => {
        }
    };
    let mut resp: String = String::with_capacity(32768 * 32);
    loop {
        let mut buf = [0; 32768 * 32];
        let _ = match socket.read(&mut buf) {
            Err(e) => {
                println!("[ERROR] on proxy: {}", e);
                return result_err;
            },
            Ok(m) => {
                let mut tmp_string = String::new();
                for byte in buf.iter() {
                    tmp_string.push(*byte as char);
                }
                let tmp_string = get_string_from_buffer_string(tmp_string);
                resp.push_str(&tmp_string);
                if end_in_two_nl(&resp) {
                    break;
                }
                m
            }
        };
    }
    resp
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

fn handle_client(mut stream: TcpStream) {
    // Sixty second connection timeout
    let timeout_time = Duration::new(60, 0);
    let _ = match stream.set_read_timeout(Option::Some(timeout_time)) {
            Err(e) => panic!("[ERROR] setting timeout: {}", e),
            Ok(m) => m
    };
	let start_time = timestamp();

	let mut from_client = String::new();

	'outer: loop {
	    'inner: loop {
	        let mut buf_tmp = [0; 1024];
	        let _ = match stream.read(&mut buf_tmp) {
	            Err(e) => {
                    println!("[ERROR] on read: {}\n |-> Probably past connection timeout.", e);
                    break 'outer;
                },
	            Ok(m) => {
                    println!("Current request time: {}", timestamp() - start_time);
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

        let local_response: String = proxy_req_to_localhost(from_client.clone());

		match stream.write(local_response.as_bytes()) {
			Err(e) => {
                println!("[ERROR] on write: {}", e);
				break 'outer;
			}
			Ok(_) => {
                println!("Response time: {}", timestamp() - start_time);
				break 'outer;
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
