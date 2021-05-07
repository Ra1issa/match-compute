// A simple single threaded example of PSI with match and compute
mod utils;
use match_compute::util;
use crate::utils::run_client::run_client;


fn main() {

    let path = util::get_path();
    let parameters = util::parse_config(&mut path.clone());
    let (address, set_size, id_size, payload_size, max_payload, _, _) = util::get_config_experiments(&parameters);

    let (time, read, written) = run_client(&address, set_size, id_size, max_payload, payload_size).unwrap();

    println!("TOTAL TIME in {} s",time);
    println!("TOTAL READ {} Mb",read);
    println!("TOTAL WRITTEN {} Mb",written);

}
