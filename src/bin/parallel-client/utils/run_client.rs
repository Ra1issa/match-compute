use match_compute::util;

use crate::utils::{
    prepare_files::prepare_files,
    client_thread::client_thread,
    join_aggregates::join_aggregates,
    test::*,
};


use std::{
    time::{Duration},
    time::SystemTime,
    thread,
};


pub fn run_client(set_size: usize, id_size: usize, max_payload:u64,
                 payload_size: usize, fake_data: bool) -> (u64, f64, f64){

    let start = SystemTime::now();
    let mut path = util::get_path();
    let parameters = util::parse_config(&mut path.clone());
    let (address, client_path, sleeptime, precision, nthread,
        megasize, client_padding, id_position, payload_position) = util::get_config_client(&parameters);

    let (ids, payloads) = if fake_data == true {
            // The ids & payloads are generated at random
            util::generate_dummy_data(set_size, id_size, max_payload)
        }else{
            // The ids & payloads are read from the csv according to their schema (column names)
            util::parse_files(id_position, payload_position, &client_path)
        };

   // Bucketize the data and split into megabins that are distributed among threads
   path.push("bin/parallel-client/data");
   let (read_init, written_init) = prepare_files(&mut path, &address, nthread, megasize,
                                                &ids, &payloads, client_padding).unwrap();

   // Wait for the server to be done
   let duration = Duration::from_secs(sleeptime);
   thread::sleep(duration);

    // Each thread handles its own megabins and speaks to the appropriate other party thread
    // via a dedicated port. The partial results of this computation are garbled and
    // stored into appropriate files. They are handled later to produce the correct output.
    let mut handle = Vec::new();
    for i in 0..nthread {
        let mut path_thread = path.clone();
        let address_thread = address.clone();
       handle.push(thread::spawn(move || {
           client_thread(&mut path_thread, &address_thread, i, payload_size).unwrap()
       }));
   }
   let mut results = Vec::new();
   for thread in handle {
        results.push(thread.join().unwrap()); // maybe consider handling errors propagated from the thread here
    }
   // The partial results are joined and the output is produced
    thread::sleep(duration);
    let (_result_cardinality, read_final, written_final) = join_aggregates(&mut path, &address, nthread, precision, payload_size).unwrap();

    let mut total_read = read_final + read_init;
    let mut total_written = written_final + written_init;
    for (r, w) in results{
        total_read = total_read + r;
        total_written = total_written + w;
    }

    println!("TOTAL TIME in {} s",start.elapsed().unwrap().as_secs());
    println!("TOTAL READ {} Mb",total_read);
    println!("TOTAL WRITTEN {} Mb",total_written);

    clear_results(&parameters,&mut path, &ids, &payloads, precision, fake_data);
    println!("Experiment done !");
    (start.elapsed().unwrap().as_secs(), total_read, total_written)
}
