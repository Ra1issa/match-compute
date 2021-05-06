use popsicle::psty_payload::{Receiver, ReceiverState};
use popsicle::psty_utils::psty_large::{
    ReceiverMegabins,
};
use scuttlebutt::{AesRng, TrackChannel, SymChannel};

use match_compute::util;
use std::{
    fs::{File},
    io::{Write, Read},
    net::{TcpStream},
    time::SystemTime,
    path::PathBuf,
    io::Error,
};

use bincode;
use serde_json;

fn client_protocol(mut channel: TrackChannel<SymChannel<TcpStream>>,
    path: &mut PathBuf, thread_id: usize, payload_size: usize)
    ->(f64, f64){
    let start = SystemTime::now();
    println!("Receiver Thread {} Starting computation", thread_id);
    let mut rng = AesRng::new();

    let mut thread_path = "thread".to_owned();
    thread_path.push_str(&thread_id.to_string());
    path.push(thread_path);

    path.push("states.txt");
    let path_str = path.clone().into_os_string().into_string().unwrap();
    let mut file_states = File::open(path_str).unwrap();
    path.pop();

    let mut buff= Vec::new();

    file_states.read_to_end(&mut buff).unwrap();

    let states: Vec<ReceiverState> = bincode::deserialize(&mut buff).unwrap();
    let nmegabins = states.len();
    let mut megabins = ReceiverMegabins{
        states,
        nmegabins,
    };

    let mut psi = Receiver::init(&mut channel, &mut rng).unwrap();
    let p =  fancy_garbling::util::primes_with_width(payload_size as u32).len() + 1;
    let (acc, sum_weights) = psi.compute_circuit(p, payload_size, &mut megabins,&mut channel, &mut rng).unwrap();

    println!(
        "Receiver Thread {} :: total circuit building & computation time: {} ms", thread_id,
        start.elapsed().unwrap().as_millis()
    );
    println!(
        "Receiver Thread {} :: total circuit building & computation communication (read): {:.2} Mb",thread_id,
        channel.kilobits_read() / 1000.0
    );
    println!(
        "Receiver Thread {} :: total circuit building & computation communication (write): {:.2} Mb",thread_id,
        channel.kilobits_written() / 1000.0
    );

    path.push("output_aggregate.txt");
    let path_str = path.clone().into_os_string().into_string().unwrap();
    let mut file_aggregate = File::create(path_str).unwrap();
    path.pop();


    path.push("output_sum_weights.txt");
    let path_str = path.clone().into_os_string().into_string().unwrap();
    let mut file_sum_weights = File::create(path_str).unwrap();
    path.pop();

    let aggregate_json = serde_json::to_string(&util::crt_to_wires(&acc)).unwrap();
    let sum_weights_json = serde_json::to_string(&util::crt_to_wires(&sum_weights)).unwrap();

    file_aggregate.write(aggregate_json.as_bytes()).unwrap();
    file_sum_weights.write(sum_weights_json.as_bytes()).unwrap();

    let total_read = channel.kilobits_read() / 1000.0;
    let total_written = channel.kilobits_written() / 1000.0;
    (total_read, total_written)
}

pub fn client_thread(path: &mut PathBuf, address: &str, thread_id: usize,
                    payload_size: usize)
    -> Result<(f64, f64), Error>{
    let port_prefix = format!("{}{}", address,":800");
    let port = format!("{}{}", port_prefix, thread_id.to_string());

    match TcpStream::connect(port) {
        Ok(stream) => {
            let channel = TrackChannel::new(SymChannel::new(stream));
            Ok(client_protocol(channel, path, thread_id, payload_size))
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
            Err(e)
        }
    }
}
