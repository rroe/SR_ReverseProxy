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

fn proxy_req_to_localhost_bytes(client_req: &[u8]) -> Option<Vec<u8>> {
    let mut socket = TcpStream::connect("127.0.0.1:80").unwrap();
    let _ = match socket.write(client_req) {
        Err(e) => {
            println!("[ERROR] on proxy: {}", e);
            return None;
        },
        Ok(m) => {
            m
        }
    };
    let mut resp: Vec<u8> = Vec::new();
    loop {
        let mut buf = [0; 32768];
        let _ = match socket.read(&mut buf) {
            Err(e) => {
                println!("[ERROR] on proxy: {}", e);
                return None;
            },
            Ok(m) => {
                for byte in buf.iter() {
                    resp.push(*byte);
                }
                break;
                m
            }
        };
    }
    Some(resp)
}

fn vec_end_in_nl(input: &Vec<u8>) -> bool {
    let len = input.len();
    let one = input[len - 1];
    let two = input[len - 2];
    let three = input[len - 3];
    let four = input[len - 4];
    if (one != 10 && one != 13) || (two != 10 && two != 13) || (three != 10 && three != 13) || (four != 10 && four != 13) {
        return false;
    }
    true
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

    let result_err: String = String::from("<html><head><title>Reverse Proxy</title></head><body>Unknown Error Occurred.</body></html>");

    let _ = match stream.set_read_timeout(Option::Some(timeout_time)) {
            Err(e) => panic!("[ERROR] setting timeout: {}", e),
            Ok(m) => m
    };

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

                    if chunk_end_in_nl(&buf_string) {
						break 'inner;
					}
	                m
	            },
	        };
	    }

        let local_response = proxy_req_to_localhost_bytes(from_client.as_bytes());

        match local_response {
            Some(vec) => {
                match stream.write(vec.as_slice()) {
        			Err(e) => {
                        println!("[ERROR] on write: {}", e);
        				break 'outer;
        			}
        			Ok(_) => {
                        println!("Finishing request.");
        				break 'outer;
        			}
        		}
            },
            None => {
                match stream.write(result_err.as_bytes()) {
        			Err(e) => {
                        println!("[ERROR] on write: {}", e);
        				break 'outer;
        			}
        			Ok(_) => {
                        println!("Finishing request.");
        				break 'outer;
        			}
        		}
            },
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
