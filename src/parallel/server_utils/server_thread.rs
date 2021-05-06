// Partial Computation per thread
use popsicle::psty_payload::{Sender, SenderState};
use popsicle::psty_utils::psty_large::{
    SenderMegabins,
};

use scuttlebutt::{AesRng, TrackChannel, SymChannel};

use fancy_garbling::{
    CrtBundle,
    Wire,
};

use std::{
    fs::{File},
    io::{Write, Read},
    net::{TcpListener, TcpStream},
    time::SystemTime,
    path::PathBuf,
};
use serde_json;
use bincode;

fn crt_to_wires(v: &[CrtBundle<Wire>])-> Vec<Vec<Wire>>{
    v.into_iter()
     .map(|c| c.wires().to_vec()).collect()
}

fn server_protocol(mut stream: TrackChannel<SymChannel<TcpStream>>, path:&mut PathBuf,
            thread_id: usize, payload_size: usize) {
    let start = SystemTime::now();
    println!("Sender Thread {} Starting computation", thread_id);

    let mut rng = AesRng::new();

    path.push("delta.txt");
    let path_delta = path.clone().into_os_string().into_string().unwrap();
    path.pop();

    let mut thread_path = "thread".to_owned();
    thread_path.push_str(&thread_id.to_string());
    path.push(thread_path);

    path.push("states.txt");
    let path_str = path.clone().into_os_string().into_string().unwrap();
    let mut file_states = File::open(path_str).unwrap();
    path.pop();

    let mut buff= Vec::new();

    file_states.read_to_end(&mut buff).unwrap();

    let states: Vec<SenderState> = bincode::deserialize(&mut buff).unwrap();
    let nmegabins = states.len();
    let mut megabins = SenderMegabins{
        states,
        nmegabins,
    };
    let mut psi = Sender::init(&mut stream, &mut rng).unwrap();
    let p =  fancy_garbling::util::primes_with_width(payload_size as u32).len() + 1;
    let (acc, sum_weights) = psi.compute_circuit(p, payload_size, &mut megabins, &path_delta, &mut stream, &mut rng).unwrap();

    println!(
        "Sender Thread {} :: total circuit building & computation time: {} ms", thread_id,
        start.elapsed().unwrap().as_millis()
    );
    println!(
        "Sender Thread {} :: total circuit building & computation communication (read): {:.2} Mb",thread_id,
        stream.kilobits_read() / 1000.0
    );
    println!(
        "Sender Thread {} :: total circuit building & computation communication (write): {:.2} Mb",thread_id,
        stream.kilobits_written() / 1000.0
    );
    path.push("output_aggregate.txt");
    let path_str = path.clone().into_os_string().into_string().unwrap();
    let mut file_aggregate = File::create(path_str).unwrap();
    path.pop();

    path.push("output_sum_weights.txt");
    let path_str = path.clone().into_os_string().into_string().unwrap();
    let mut file_sum_weights = File::create(path_str).unwrap();
    path.pop();

    let aggregate_json = serde_json::to_string(&crt_to_wires(&acc)).unwrap();
    let sum_weights_json = serde_json::to_string(&crt_to_wires(&sum_weights)).unwrap();

    file_aggregate.write(aggregate_json.as_bytes()).unwrap();
    file_sum_weights.write(sum_weights_json.as_bytes()).unwrap();
}

pub fn server_thread(path:&mut PathBuf, address: &str, thread_id: usize, payload_size: usize) {
    let port_prefix = format!("{}{}", address,":800");
    let port = format!("{}{}", port_prefix, thread_id.to_string());
    println!("Server listening on {}", port);

    let listener = TcpListener::bind(port).unwrap();
    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                let channel = TrackChannel::new(SymChannel::new(stream));
                server_protocol(channel, path, thread_id, payload_size);
                return;
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    drop(listener);
}
