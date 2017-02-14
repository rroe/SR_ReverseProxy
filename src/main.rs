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

const bad: Vec<u8> = vec![60, 104, 116, 109, 108, 62, 60, 104, 101, 97, 100, 62, 60, 116, 105, 116,
                108, 101, 62, 82, 101, 118, 101, 114, 115, 101, 32, 80, 114, 111,
                120, 121, 60, 47, 116, 105, 116, 108, 101, 62, 60, 47, 104, 101, 97,
                100, 62, 60, 98, 111, 100, 121, 62, 85, 110, 107, 110, 111, 119, 110,
                32, 69, 114, 114, 111, 114, 32, 79, 99, 99, 117, 114, 114, 101, 100, 46,
                60, 47, 98, 111, 100, 121, 62, 60, 47, 104, 116, 109, 108, 62];

fn proxy_req_to_localhost_bytes(client_req: &[u8]) -> &[u8] {
    let result_err: String = String::from("<html><head><title>Reverse Proxy</title></head><body>Unknown Error Occurred.</body></html>");

    let mut socket = TcpStream::connect("127.0.0.1:80").unwrap();
    let _ = match socket.write(client_req) {
        Err(e) => {
            println!("[ERROR] on proxy: {}", e);
            return bad.as_slice();
        },
        Ok(m) => {
            m
        }
    };
    let mut resp: Vec<u8> = Vec::new();
    loop {
        let mut buf = [0; 32768 * 32];
        let _ = match socket.read(&mut buf) {
            Err(e) => {
                println!("[ERROR] on proxy: {}", e);
                return result_err.as_bytes();
            },
            Ok(m) => {
                for byte in buf.iter() {
                    resp.push(*byte);
                }
                // if vec_end_in_nl(&resp) {
                //     break;
                // }
                break;
                m
            }
        };
    }
    resp.as_slice()
}

fn vec_end_in_nl(input: &Vec<u8>) -> bool {
    let len = input.len();
    let one = input[len - 1];
    let two = input[len - 2];
    let three = input[len - 3];
    let four = input[len - 4];
    if (one != 10 && one != 13) || (two != 10 && two != 13) || (three != 10 && three != 13) || (four != 10 && four != 13) {
        println!("{} {} {} {}", one, two, three, four);
        return false;
    }
    true
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
            m
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
                // print_chunk("FROM_LOC", &tmp_string);
                let tmp_string = get_string_from_buffer_string(tmp_string);
                resp.push_str(&tmp_string);
                if end_in_two_nl(&resp) {
                    // the below *seems* to be slower here - I'll need to restart my computer to make sure it isn't a memory issue
                // if chunk_end_in_nl(&resp) {
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

// Hopefully much faster than the above one - if it doesn't end in garbage null bytes
fn chunk_end_in_nl(string: &String) -> bool {
    let last_four: Vec<char> = string.chars().rev().take(4).collect();
    for i in 0..4 {
        if (last_four[i] as u8) != 10 && (last_four[i] as u8) != 13 {
            return false;
        }
    }
    true
}

fn print_chunk(title: &str, string: &String) {
    println!("=============== [ START {} ] ============== \n{}\n============= [ END ] ==============", title, string);
}

fn handle_client(mut stream: TcpStream, my_num: u32) {
    let timeout_time = Duration::new(60 * 5, 0);
    let _ = match stream.set_read_timeout(Option::Some(timeout_time)) {
            Err(e) => panic!("[ERROR] setting timeout: {}", e),
            Ok(m) => m
    };
	// let start_time = timestamp();

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
                    //println!("Chunk for {} time: {} sec", my_num, timestamp() - start_time);
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
                    // print_chunk("RECEIVED", &s);
					let buf_string = get_string_from_buffer_string(s);
					from_client.push_str(&buf_string);

					// if end_in_two_nl(&from_client) == true {
                    if chunk_end_in_nl(&buf_string) {
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
                //println!("Request #{} time: {} sec", my_num, timestamp() - start_time);
                println!("Finishing request.");
				break 'outer;
			}
		}
	}
}

fn main() {
    let listener = TcpListener::bind("127.0.0.1:88").unwrap();
	println!("Starting TCP listener...");
    let mut count: u32 = 0;
    for stream in listener.incoming() {
    	match stream {
            Err(e) => { println!("Failed: {}", e) }
            Ok(stream) => {
                count = count + 1;
                thread::spawn(move || {
                    handle_client(stream, count)
                });
            }
        }
    }
}
