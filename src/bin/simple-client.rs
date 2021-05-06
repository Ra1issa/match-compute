// A simple single threaded example of PSI with match and compute

use popsicle::psty_payload::{Receiver};

use rand::{CryptoRng, Rng};
use scuttlebutt::{AesRng, Block512, TrackChannel, SymChannel};

use std::net::{TcpStream};


pub fn int_vec_block512(values: Vec<u64>) -> Vec<Block512> {
    values.into_iter()
          .map(|item|{
            let value_bytes = item.to_le_bytes();
            let mut res_block = [0 as u8; 64];
            for i in 0..8{
                res_block[i] = value_bytes[i];
            }
            Block512::from(res_block)
         }).collect()
}

pub fn rand_u64_vec<RNG: CryptoRng + Rng>(n: usize, _modulus: u64, _rng: &mut RNG) -> Vec<u64>{
    (0..n).map(|_| 100).collect()
    // rng.gen::<u64>()%modulus
}

pub fn enum_ids(n: usize, id_size: usize) ->Vec<Vec<u8>>{
    let mut ids = Vec::with_capacity(n);
    for i in 0..n as u64{
        let v:Vec<u8> = i.to_le_bytes().iter().take(id_size).cloned().collect();
        ids.push(v);
    }
    ids
}

fn client_protocol(mut stream: TrackChannel<SymChannel<TcpStream>>,){
    const ITEM_SIZE: usize = 16;
    const SET_SIZE: usize = 10000;
    let payload_size: usize = 64;

    let mut rng = AesRng::new();
    let receiver_inputs = enum_ids(SET_SIZE, ITEM_SIZE);
    let payloads = int_vec_block512(rand_u64_vec(SET_SIZE, u64::pow(10,6), &mut rng));

    let mut psi = Receiver::init(&mut stream, &mut rng).unwrap();
    // For small to medium sized sets where batching can occur accross all bins
    let weighted_mean = psi
        .full_protocol(&receiver_inputs, &payloads, payload_size, &mut stream, &mut rng)
        .unwrap();
}

fn main() {
    match TcpStream::connect("0.0.0.0:3000") {
        Ok(stream) => {
            let channel = TrackChannel::new(SymChannel::new(stream));
            client_protocol(channel);
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
        }
    }
    println!("Terminated.");
}
