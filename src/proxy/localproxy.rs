extern crate time;
extern crate rand;

use std::net::{TcpListener, TcpStream, Shutdown};
use std::io::Read;
use std::time::Duration;
use std::io::Write;
use rand::Rng;

use proxy::buffer;

pub fn handle_client(mut stream: TcpStream, prob_curve: Vec<u64>) {
    let rand_num = rand::thread_rng().gen_range(0, prob_curve.len());

    // Set the timeout to a random time based on our probability curve
    let timeout_time = Duration::new(prob_curve[rand_num] + 1, 0);
    // println!("Setting timeout time for req to {}s", prob_curve[rand_num]);

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
					let buf_string = buffer::get_string_from_buffer_string(s);
					from_client.push_str(&buf_string);

                    if buffer::chunk_end_in_nl(&buf_string) {
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
    stream.shutdown(Shutdown::Both).expect("Failed to shutdown local proxy stream.");
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
    socket.shutdown(Shutdown::Both).expect("Failed to shutdown local proxy stream.");
    Some(resp)
}
