mod utils;
use match_compute::util;

use crate::utils::run_client::run_client;

pub fn main(){
    let set_size: usize = 5;
    let id_size: usize = 16;
    let max_payload: u64 = 1000;
    let payload_size: usize = 64;
    let fake_data: bool = true;

    let (time, read, written) = run_client(set_size, id_size, max_payload, payload_size, fake_data);
}
