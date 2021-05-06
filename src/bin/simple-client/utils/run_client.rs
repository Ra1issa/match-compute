// A simple single threaded example of PSI with match and compute

use match_compute::util;
use popsicle::psty_payload::{Receiver};

use scuttlebutt::{AesRng, TrackChannel, SymChannel};

use std::net::{TcpStream};

fn client_protocol(set_size: usize, id_size: usize, max_payload: u64, payload_size: usize,
                    mut stream: TrackChannel<SymChannel<TcpStream>>){

    let mut rng = AesRng::new();
    let (receiver_inputs, payloads) = util::generate_dummy_data(set_size, id_size, max_payload);

    let mut psi = Receiver::init(&mut stream, &mut rng).unwrap();
    // For small to medium sized sets where batching can occur accross all bins
    let _weighted_mean = psi
        .full_protocol(&receiver_inputs, &payloads, payload_size, &mut stream, &mut rng)
        .unwrap();
}

pub fn run_client(set_size: usize, id_size: usize, max_payload: u64, payload_size: usize){
    match TcpStream::connect("0.0.0.0:3000") {
        Ok(stream) => {
            let channel = TrackChannel::new(SymChannel::new(stream));
            client_protocol(set_size, id_size, max_payload, payload_size, channel);
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Terminated.");
}
