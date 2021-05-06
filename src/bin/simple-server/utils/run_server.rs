// A simple single threaded example of PSI with match and compute
use match_compute::util;
use popsicle::psty_payload::{Sender};

use scuttlebutt::{AesRng, TrackChannel, SymChannel};

use std::{
    net::{TcpListener, TcpStream},
};

fn server_protocol(set_size: usize, id_size: usize, max_payload: u64, payload_size: usize,
                    mut stream: TrackChannel<SymChannel<TcpStream>>){

    let mut rng = AesRng::new();
    let (sender_inputs, payload) = util::generate_dummy_data(set_size, id_size, max_payload);
    let mut psi = Sender::init(&mut stream, &mut rng).unwrap();

    let _ = psi
        .full_protocol(&sender_inputs, &payload, payload_size, &mut stream, &mut rng)
        .unwrap();
}


pub fn run_server(set_size: usize, id_size: usize, max_payload: u64, payload_size: usize){
    let listener = TcpListener::bind("0.0.0.0:3000").unwrap();
    // accept connections and process them, spawning a new thread for each one
    println!("Server listening on port 3000");
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                    let channel = TrackChannel::new(SymChannel::new(stream));
                    server_protocol(set_size, id_size, max_payload, payload_size, channel);
                    return;

            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    drop(listener);
}
