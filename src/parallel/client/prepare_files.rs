// Bucketize Data and Seperate it among threads
use popsicle::psty_payload::{Receiver, ReceiverState};

use scuttlebutt::{AesRng, Block512, TrackChannel, SymChannel};
use rand::{CryptoRng, Rng};
use std::{
    fs::{File, create_dir_all},
    io::{Write},
    net::{TcpStream},
    time::SystemTime,
    path::PathBuf,
    collections::HashMap,
    io::Error,
};

use bincode;

fn pad_data<RNG: CryptoRng + Rng>(ids: &[Vec<u8>], payloads: &[Block512], client_padding: usize, rng: &mut RNG) -> (Vec<Vec<u8>>, Vec<Block512>){

    let mut real_data = HashMap::new();
    for i in 0..ids.len(){
        let mut id = [0 as u8; 8];
        for j in 0..8{
            id[j] = ids[i][j];
        }
        real_data.insert(u64::from_le_bytes(id), payloads[i]);
    }
    let mut ids_padded = ids.to_vec();
    let mut payloads_padded = payloads.to_vec();

    for _i in 0..client_padding{
        let mut new_id = rng.gen::<u64>();
        while real_data.contains_key(&new_id){
            new_id = rng.gen::<u64>();
        }
        ids_padded.push(new_id.to_le_bytes().to_vec());
        payloads_padded.push(Block512::from([0 as u8; 64]));
    }
    (ids_padded, payloads_padded)
}


fn client_protocol(mut channel: TrackChannel<SymChannel<TcpStream>>, path: &mut PathBuf, nthread: usize,
                    megasize: usize, ids: &[Vec<u8>], payloads: &[Block512], client_padding: usize)
                    ->(f64, f64){
    let start = SystemTime::now();

    let mut rng = AesRng::new();

    let (ids_pad, payloads_pad) = pad_data(ids, payloads, client_padding, &mut rng);

    // The Receiver bucketizes the data and seperates into megabins during the cuckoo hashing.
    // And sends the number of megabins, number of bins etc. to the sender
    let mut psi = Receiver::init(&mut channel, &mut rng).unwrap();
    let megabins = psi.bucketize_data_large(&ids_pad, &payloads_pad, megasize, &mut channel, &mut rng).unwrap();

    let megabin_per_thread = ((megabins.nmegabins as f32)/(nthread as f32)).ceil() as usize;

    println!("Number of megabins per thread {:?}", megabin_per_thread);

    let states_per_thread:Vec<&[ReceiverState]> = megabins.states.chunks(megabin_per_thread).collect();
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
        "Receiver :: Bucketization time : {} ms",
        start.elapsed().unwrap().as_millis()
    );
    println!(
        "Receiver ::Bucketization time (read): {:.2} Mb",
        channel.kilobits_read() / 1000.0
    );
    println!(
        "Receiver :: Bucketization time (write): {:.2} Mb",
        channel.kilobits_written() / 1000.0
    );

    let total_read = channel.kilobits_read() / 1000.0;
    let total_written = channel.kilobits_written() / 1000.0;
    (total_read, total_written)
}

pub fn prepare_files(path: &mut PathBuf, address: &str, nthread: usize, megasize: usize,
                    ids: &[Vec<u8>], payloads: &[Block512], client_padding: usize)
                    -> Result<(f64, f64), Error>{
    let address = format!("{}{}", address,":3000");

    match TcpStream::connect(address) {
        Ok(stream) => {
            let channel = TrackChannel::new(SymChannel::new(stream));
            Ok(client_protocol(channel, path, nthread, megasize, ids, payloads, client_padding))
        },
        Err(e) => {
            println!("Failed to connect: {}", e);
            Err(e)
        }
    }
}
