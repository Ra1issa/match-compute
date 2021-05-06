// A simple single threaded example of PSI with match and compute
mod utils;
use match_compute::util;
use crate::utils::run_server::run_server;

pub fn main(){
    let path = util::get_path();
    let parameters = util::parse_config(&mut path.clone());
    let (address, set_size, id_size, payload_size, max_payload, trials, _) = util::get_config_experiments(&parameters);

    for _i in 0..trials{
        run_server(&address, set_size, id_size, max_payload, payload_size);
    }
}
