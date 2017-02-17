extern crate rand;

use std::thread;
use rand::Rng;
use std::net::{TcpListener, TcpStream, Shutdown};

mod proxy;

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

fn main() {
    let listener = TcpListener::bind(":::88").unwrap();
	println!("Starting TCP listener...");
    let mut prob_curve: Vec<u64> = Vec::new();
    generate_linear(15, &mut prob_curve);
    for stream in listener.incoming() {
    	match stream {
            Err(e) => { println!("Failed: {}", e) }
            Ok(stream) => {
                let copy = prob_curve.clone();
                thread::spawn(move || {
                    proxy::localproxy::handle_client(stream, copy)
                });
            }
        }
    }
}
