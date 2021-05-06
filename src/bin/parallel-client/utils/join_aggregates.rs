use popsicle::psty_payload::{Receiver};
use match_compute::util;

use scuttlebutt::{AesRng, SymChannel, TrackChannel};

use std::{
    fs::{File, write, read_to_string},
    net::{TcpStream},
    time::SystemTime,
    io::Error,
    path::PathBuf,
};
use serde_json;


fn client_protocol(mut channel: TrackChannel<SymChannel<TcpStream>>,
    path:&mut PathBuf, nthreads: usize, _precision: u32, payload_size: usize)
    -> (u128, f64, f64){
    let start = SystemTime::now();
    let mut rng = AesRng::new();

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

        aggregates.append(&mut util::wires_to_crt(&partial_aggregate));
        sum_weights.append(&mut util::wires_to_crt(&partial_sum_weights));

        path.pop();
    }

    let mut psi = Receiver::init(&mut channel, &mut rng).unwrap();
    let p =  fancy_garbling::util::primes_with_width(payload_size as u32).len() + 1;
    let weighted_mean = psi.join_circuits(p, &mut aggregates,
                            &mut sum_weights, &mut channel,&mut rng).unwrap();
    println!("weighted_mean: {:?}", weighted_mean);


    path.pop();
    path.push("result.txt");
    let path_str = path.clone().into_os_string().into_string().unwrap();
    path.pop();

    let _ = File::create(path_str.clone()).unwrap();

    let mut output_write = "Weighted Mean: ".to_owned();
    output_write.push_str(&weighted_mean.to_string());

    write(path_str, output_write).expect("Unable to write file");

    println!(
        "Receiver :: total Joining threads results time: {} ms",
        start.elapsed().unwrap().as_millis()
    );
    println!(
        "Receiver :: total Joining threads results time (read): {:.2} Mb",
        channel.kilobits_read() / 1000.0
    );
    println!(
        "Receiver :: total Joining threads results time  (write): {:.2} Mb",
        channel.kilobits_written() / 1000.0
    );

    let total_read = channel.kilobits_read() / 1000.0;
    let total_written = channel.kilobits_written() / 1000.0;
    (weighted_mean, total_read, total_written)
}

pub fn join_aggregates(path:&mut PathBuf, address: &str,
    nthreads: usize, precision: u32, payload_size: usize)
    -> Result<(u128, f64, f64), Error>{
    let port_prefix = format!("{}{}", address,":3000");

    match TcpStream::connect(port_prefix) {
        Ok(stream) => {
            let channel = TrackChannel::new(SymChannel::new(stream));
            Ok(client_protocol(channel, path, nthreads, precision, payload_size))
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
            Err(e)
        }
    }
}
