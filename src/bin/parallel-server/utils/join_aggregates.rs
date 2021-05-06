use popsicle::psty_payload::{Sender};

use fancy_garbling::{
    CrtBundle,
    Wire,
};
use scuttlebutt::{AesRng, SymChannel, TrackChannel};

use std::{
    fs::{read_to_string},
    net::{TcpListener, TcpStream},
    time::SystemTime,
    path::PathBuf,
};
use serde_json;


fn wires_to_crt(v: &[Vec<Wire>])-> Vec<CrtBundle<Wire>>{
    v.into_iter()
     .map(|c| CrtBundle::new(c.to_vec())).collect()
}


fn server_protocol(mut channel: TrackChannel<SymChannel<TcpStream>>, path:&mut PathBuf, nthreads: usize) {
    let start = SystemTime::now();
    let mut rng = AesRng::new();

    path.push("delta.txt");
    let path_delta = path.clone().into_os_string().into_string().unwrap();
    path.pop();

    let mut aggregates= Vec::new();
    let mut sum_weights= Vec::new();
    for thread_id in 0..nthreads{
        let mut thread_path = "thread".to_owned();
        thread_path.push_str(&thread_id.to_string());
        path.push(thread_path);

        path.push("output_aggregate.txt");
        let path_str = path.clone().into_os_string().into_string().unwrap();
        let partial_aggregate: Vec<Vec<Wire>> = serde_json::from_str(&read_to_string(path_str).unwrap()).unwrap();
        path.pop();

        path.push("output_sum_weights.txt");
        let path_str = path.clone().into_os_string().into_string().unwrap();
        let partial_sum_weights: Vec<Vec<Wire>> = serde_json::from_str(&read_to_string(path_str).unwrap()).unwrap();
        path.pop();


        aggregates.append(&mut wires_to_crt(&partial_aggregate));
        sum_weights.append(&mut wires_to_crt(&partial_sum_weights));

        path.pop();
    }

    let mut psi = Sender::init(&mut channel, &mut rng).unwrap();
    let _ = psi.join_circuits(&mut aggregates, &mut sum_weights, &path_delta, &mut channel,&mut rng);

    println!(
        "Sender :: total Joining threads results time: {} ms",
        start.elapsed().unwrap().as_millis()
    );
    println!(
        "Sender :: total Joining threads results time (read): {:.2} Mb",
        channel.kilobits_read() / 1000.0
    );
    println!(
        "Sender :: total Joining threads results time  (write): {:.2} Mb",
        channel.kilobits_written() / 1000.0
    );
}

pub fn join_aggregates(path:&mut PathBuf, address: &str, nthreads: usize) {
    let port_prefix = format!("{}{}", address,":3000");
    println!("Server listening on {}", port_prefix);
    let listener = TcpListener::bind(port_prefix).unwrap();


    for stream in listener.incoming() {
        match stream {
            Ok(stream) => {
                println!("New connection: {}", stream.peer_addr().unwrap());
                let channel = TrackChannel::new(SymChannel::new(stream));
                server_protocol(channel, path, nthreads);
                return;
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
    drop(listener);
}
