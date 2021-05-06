mod utils;
use match_compute::util;

use crate::utils::run_server::run_server;

pub fn main(){
    let set_size: usize = 5;
    let id_size: usize = 16;
    let max_payload: u64 = 1000;
    let payload_size: usize = 64;
    let fake_data: bool = true;

    run_server(set_size, id_size, max_payload, payload_size, fake_data);

    println!("Experiments done !");
}
