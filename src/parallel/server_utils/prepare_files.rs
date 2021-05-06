// Bucketize Data and Seperate it among threads
use popsicle::psty_payload::{Sender, SenderState};

use scuttlebutt::{AesRng, Block512, TrackChannel, SymChannel};
extern crate fancy_garbling;
use fancy_garbling::Wire;

use std::{
    fs::{File, create_dir_all},
    io::{Write},
    net::{TcpListener, TcpStream},
    collections::HashMap,
    time::SystemTime,
    path::PathBuf,
};
use bincode;
use serde_json;

pub fn generate_deltas() -> HashMap<u16, Wire> {
    let mut deltas = HashMap::new();
    let mut rng = rand::thread_rng();
    for q in 2..255{
        deltas.insert(q, Wire::rand_delta(&mut rng, q));
    }
    deltas
}

fn server_protocol(mut stream: TrackChannel<SymChannel<TcpStream>>, path: &mut PathBuf, nthread: usize,
                    ids: &[Vec<u8>], payloads: &[Block512], payload_size: usize){
    let start = SystemTime::now();

    let mut rng = AesRng::new();
    let deltas = generate_deltas();

    path.push("delta.txt");
    let path_deltas = path.clone().into_os_string().into_string().unwrap();
    let mut file_deltas = File::create(path_deltas).unwrap();
    path.pop();

    let deltas_json = serde_json::to_string(&deltas).unwrap();
    file_deltas.write(deltas_json.as_bytes()).unwrap();

    let mut psi = Sender::init(&mut stream, &mut rng).unwrap();

    // At the sender side, the data is bucketized using simple hashing but is not immediately
    // divided into megabins (contrary to the receiver)
    let megabins = psi.bucketize_data_large(
                    &ids, &payloads, payload_size, &mut stream, &mut rng
                ).unwrap();

    let megabin_per_thread = ((megabins.nmegabins as f32)/(nthread as f32)).ceil() as usize;


    // After bucketization, data is divided accross threads at this level according to the
    // megabin size that the client sends out
    let states_per_thread:Vec<&[SenderState]> = megabins.states.chunks(megabin_per_thread).collect();

    // Create files and folders with the data that each thread should handle.
    for i in 0 ..nthread{
        let mut thread_path = "thread".to_owned();
        thread_path.push_str(&i.to_string());
        path.push(thread_path);

        let path_str = path.clone().into_os_string().into_string().unwrap();
        let _ = create_dir_all(path_str);

        path.push("states.txt");
        let path_str = path.clone().into_os_string().into_string().unwrap();
        let mut file_states = File::create(path_str).unwrap();
        path.pop();

        let state_json = bincode::serialize(&states_per_thread[i]).unwrap();
        file_states.write(&state_json).unwrap();

        path.pop();
    }

    println!(
        "Sender :: Bucketization time: {} ms",
        start.elapsed().unwrap().as_millis()
    );
    println!(
        "Sender :: Bucketization time (read): {:.2} Mb",
        stream.kilobits_read() / 1000.0
    );
    println!(
        "Sender :: Bucketization time  (write): {:.2} Mb",
        stream.kilobits_written() / 1000.0
    );

}

pub fn prepare_files(path: &mut PathBuf, address: &str, nthread: usize,
    ids: &[Vec<u8>], payloads: &[Block512], payload_size: usize) {
    let address = format!("{}{}", address,":3000");
    println!("Server listening on {}", address);
    let listener = TcpListener::bind(address).unwrap();

    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                    let channel = TrackChannel::new(SymChannel::new(stream));
                    server_protocol(channel, path, nthread, ids, payloads, payload_size);
                    return;

            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    drop(listener);
}
