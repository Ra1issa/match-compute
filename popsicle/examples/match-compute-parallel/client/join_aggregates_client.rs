use popsicle::psty_payload::{Receiver};

use fancy_garbling::{
    Wire,
};
use scuttlebutt::{AesRng, TcpChannel};

use std::{
    fs::{File, read_to_string},
    io::Write,
    net::{TcpStream},
    time::SystemTime,
    io::Error,
    path::PathBuf,
};
use serde_json;

fn client_protocol(mut channel: TcpChannel<TcpStream>, path:&mut PathBuf, nthreads: usize) -> (u64, u64){
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
    let output = aggregate as f64 / cardinality as f64;
    println!("aggregate: {:?}", aggregate);
    println!("cardinality: {:?}", cardinality);
    println!("average: {:?}", output);

    path.push("result.txt");
    let path_str = path.clone().into_os_string().into_string().unwrap();
    path.pop();

    let mut file_result = File::create(path_str).unwrap();
    file_result.write(&output.to_le_bytes()).unwrap();


    println!(
        "Receiver :: Joining threads results time: {} ms",
        start.elapsed().unwrap().as_millis()
    );
    println!(
        "Receiver :: Joining threads results time (read): {:.2} Mb",
        channel.kilobits_read() / 1000.0
    );
    println!(
        "Receiver :: Joining threads results time  (write): {:.2} Mb",
        channel.kilobits_written() / 1000.0
    );

    (aggregate, cardinality)
}

pub fn join_aggregates(path:&mut PathBuf, address: &str, nthreads: usize) -> Result<(u64, u64), Error>{
    let port_prefix = format!("{}{}", address,":3000");

    match TcpStream::connect(port_prefix) {
        Ok(stream) => {
            let channel = TcpChannel::new(stream);
            Ok(client_protocol(channel, path, nthreads))
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
            Err(e)
        }
    }
}
