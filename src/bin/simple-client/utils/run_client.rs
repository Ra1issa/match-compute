// A simple single threaded example of PSI with match and compute

use match_compute::util;
use popsicle::psty_payload::{Receiver};

use scuttlebutt::{AesRng, TrackChannel, SymChannel};

use std::{
    net::{TcpStream},
    io::Error,
    time::SystemTime,
};

fn client_protocol(set_size: usize, id_size: usize, max_payload: u64, payload_size: usize,
                    mut channel: TrackChannel<SymChannel<TcpStream>>)-> (u128, f64, f64){
    let start = SystemTime::now();
    let mut rng = AesRng::new();
    let (receiver_inputs, payloads) = util::generate_dummy_data(set_size, id_size, max_payload);

    let mut psi = Receiver::init(&mut channel, &mut rng).unwrap();
    // For small to medium sized sets where batching can occur accross all bins
    let _weighted_mean = psi
        .full_protocol(&receiver_inputs, &payloads, payload_size, &mut channel, &mut rng)
        .unwrap();
    (start.elapsed().unwrap().as_millis(), channel.kilobits_read() / 1000.0, channel.kilobits_written() / 1000.0)
}

pub fn run_client(address: &str, set_size: usize, id_size: usize, max_payload: u64, payload_size: usize)
        ->Result<(u128, f64, f64), Error>{
    let address = format!("{}{}", address,":3000");
    match TcpStream::connect(address) {
        Ok(stream) => {
            let channel = TrackChannel::new(SymChannel::new(stream));
            Ok(client_protocol(set_size, id_size, max_payload, payload_size, channel))
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
            Err(e)
        }
    }
}
