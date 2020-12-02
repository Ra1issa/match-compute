use popsicle::psty_payload::{Receiver};

use fancy_garbling::{
    Wire,
};
use scuttlebutt::{AesRng, TcpChannel};

use std::{
    fs::{File, write, read_to_string},
    io::Write,
    net::{TcpStream},
    time::SystemTime,
    io::Error,
    path::PathBuf,
};
use serde_json;

fn client_protocol(mut channel: TcpChannel<TcpStream>, path:&mut PathBuf, nthreads: usize, precision: u32) -> (f64, u64){
    let start = SystemTime::now();
    let mut rng = AesRng::new();

    let mut aggregates= Vec::new();
    let mut cardinality= Vec::new();
    for thread_id in 0..nthreads{
        let mut thread_path = "thread".to_owned();
        thread_path.push_str(&thread_id.to_string());
        path.push(thread_path);

        path.push("output_aggregate.txt");
        let path_str = path.clone().into_os_string().into_string().unwrap();
        let partial_aggregate: Vec<Wire> = serde_json::from_str(&read_to_string(path_str).unwrap()).unwrap();
        path.pop();

        path.push("output_cardinality.txt");
        let path_str = path.clone().into_os_string().into_string().unwrap();
        let partial_cardinality: Vec<Wire> = serde_json::from_str(&read_to_string(path_str).unwrap()).unwrap();
        path.pop();

        aggregates.push(partial_aggregate);
        cardinality.push(partial_cardinality);

        path.pop();
    }

    let mut psi = Receiver::init(&mut channel, &mut rng).unwrap();
    let (aggregate, cardinality) = psi.compute_aggregates(aggregates, cardinality, &mut channel,&mut rng).unwrap();
    let aggregate: f64 = aggregate as f64/ 10_u64.pow(precision) as f64;
    let output: f64 = aggregate / cardinality as f64;

    println!("aggregate: {:?}", aggregate);
    println!("cardinality: {:?}", cardinality);
    println!("average: {:?}", output);

    path.pop();
    path.push("result.txt");
    let path_str = path.clone().into_os_string().into_string().unwrap();
    path.pop();

    let _ = File::create(path_str.clone()).unwrap();
    // file_result.write(&aggregate.to_le_bytes()).unwrap();
    // file_result.write(&cardinality.to_le_bytes()).unwrap();
    // file_result.write(&output.to_le_bytes()).unwrap();

    let mut output_write = "Aggregate: ".to_owned();
    output_write.push_str(&aggregate.to_string());
    output_write.push_str(&"\nCardinality: ".to_owned());
    output_write.push_str(&cardinality.to_string());
    output_write.push_str(&"\nAverage: ".to_owned());
    output_write.push_str(&output.to_string());

    write(path_str, output_write).expect("Unable to write file");
    // write(path_str.clone(), cardinality_str).expect("Unable to write file");
    // write(path_str.clone(), aggregate_str).expect("Unable to write file");


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

    (aggregate, cardinality)
}

pub fn join_aggregates(path:&mut PathBuf, address: &str, nthreads: usize, precision: u32) -> Result<(f64, u64), Error>{
    let port_prefix = format!("{}{}", address,":3000");

    match TcpStream::connect(port_prefix) {
        Ok(stream) => {
            let channel = TcpChannel::new(stream);
            Ok(client_protocol(channel, path, nthreads, precision))
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
            Err(e)
        }
    }
}
