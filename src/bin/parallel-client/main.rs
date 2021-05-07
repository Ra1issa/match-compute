mod utils;
use match_compute::util;
use crate::utils::run_client::run_client;

pub fn main(){
    let path = util::get_path();
    let parameters = util::parse_config(&mut path.clone());
    let (_, set_size, id_size, payload_size, max_payload, _, fake_data) = util::get_config_experiments(&parameters);

    let (time, read, written) = run_client(set_size, id_size, max_payload, payload_size, fake_data);

}
