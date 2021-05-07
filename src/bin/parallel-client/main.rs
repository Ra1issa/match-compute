mod utils;
use match_compute::util;
use crate::utils::run_client::run_client;

pub fn main(){
    let path = util::get_path();
    let parameters = util::parse_config(&mut path.clone());
    let (_, set_size, id_size, payload_size, max_payload, trials, fake_data) = util::get_config_experiments(&parameters);

    let mut times = Vec::new();
    let mut reads = Vec::new();
    let mut writes = Vec::new();
    for _i in 0..trials{
        println!("EXPERIMENT {} OUT OF {}", i+1, trials);
        let (time, read, written) = run_client(set_size, id_size, max_payload, payload_size, fake_data);
        times.push(time);
        reads.push(read);
        writes.push(written);
    }

    let average_time = times.into_iter().reduce(|a, b| a + b).unwrap()/ trials;
    let average_written = writes.into_iter() .reduce(|a, b| a + b).unwrap()/trials as f64;
    let average_read = reads.into_iter().reduce(|a, b| a + b).unwrap()/trials as f64;
    let average_total_com = (average_written + average_read)/8.0;

   println!("******Multi Threaded Experiment Measurments******");
   println!("Set Size {:?}, Payload Size {:?}, Item_Size {:?}", set_size, payload_size, id_size);
   println!("Average computation time in {:?} trials is {:?} MB", trials, average_time);
   println!("Average total communication in {:?} trials is {:?} MB", trials, average_total_com);
}
