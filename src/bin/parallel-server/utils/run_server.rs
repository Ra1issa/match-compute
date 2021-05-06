use match_compute::util;

use crate::utils::{
    prepare_files::prepare_files,
    server_thread::server_thread,
    join_aggregates::join_aggregates,
};

use popsicle::psty_utils::{
    util::parse_files,
};

use std::{
    env,
    fs::{File},
    io::{BufRead, BufReader},
    collections::HashMap,
    thread,
};
pub fn run_server(set_size: usize, id_size: usize, max_payload:u64, payload_size: usize, fake_data: bool){

    let mut path = util::get_path();
    let parameters = util::parse_config(&mut path.clone());
    let (address, server_path, nthread, id_position, payload_position) =
                                        util::get_config_sever(&parameters);

    let mut ids = Vec::new();
    let mut payloads = Vec::new();
    if fake_data == true {
        // The ids & payloads are generated at random
        println!("hello server");
        let (id, payload) = util::generate_dummy_data(set_size, id_size, max_payload);
        ids = id;
        payloads = payload;
    }else{
        // The ids & payloads are read from the csv according to their schema (column names)
        let (id, payload) = util::parse_files(id_position, payload_position, &server_path);
        ids = id;
        payloads = payload;
    }

   // Bucketize the data and split into megabins that are distributed among threads
    path.push("bin/parallel-server/data");
    prepare_files(&mut path, &address, nthread, &ids, &payloads, payload_size);

    // Each thread handles its own megabins and speaks to the appropriate other party thread
    // via a dedicated port. The partial results of this computation are garbled and
    // stored into appropriate files. They are handled later to produce the correct output.
    let mut handle = Vec::new();
    for i in 0..nthread {
        let mut path_thread = path.clone();
        let address_thread = address.clone();
       handle.push(thread::spawn(move || {
           server_thread(&mut path_thread, &address_thread, i, payload_size);
       }));
   }
   for thread in handle {
        let _ = thread.join();
    }

    // The partial results are joined and the output is produced
    join_aggregates(&mut path, &address, nthread);

    println!("Experiments done !");
}
