extern crate time;
extern crate rand;

use std::net::{TcpListener, TcpStream};
use std::thread;
use std::io::Read;
use std::time::Duration;
use std::io::Write;
use rand::Rng;

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

fn print_chunk(title: &str, string: &String) {
    println!("=============== [ START {} ] ============== \n{}\n============= [ END ] ==============", title, string);
}

fn generate_linear(size: u64, vec: &mut Vec<u64>) {
    for x in 0..size {
        for y in 0..x {
            vec.push(x);
        }
    }
}

fn generate_hyperbolic(size: u64, vec: &mut Vec<u64>) {
    for x in 0..size {
        for y in 0..x {
            vec.push(x * x);
        }
    }
}

fn handle_client(mut stream: TcpStream, prob_curve: Vec<u64>) {
    let rand_num = rand::thread_rng().gen_range(0, prob_curve.len());

    // Set the timeout to a random time based on our probability curve
    let timeout_time = Duration::new(prob_curve[rand_num] + 1, 0);
    println!("Setting timeout time for req to {}s", prob_curve[rand_num]);

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
    let listener = TcpListener::bind(":::88").unwrap();
	println!("Starting TCP listener...");
    let mut prob_curve: Vec<u64> = Vec::new();
    generate_linear(60, &mut prob_curve);
    for stream in listener.incoming() {
    	match stream {
            Err(e) => { println!("Failed: {}", e) }
            Ok(stream) => {
                let copy = prob_curve.clone();
                thread::spawn(move || {
                    handle_client(stream, copy)
                });
            }
        }
    }
}
