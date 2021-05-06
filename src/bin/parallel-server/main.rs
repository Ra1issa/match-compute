mod utils;
use match_compute::util;
use crate::utils::run_server::run_server;

pub fn main(){
    let path = util::get_path();
    let parameters = util::parse_config(&mut path.clone());
    let (_, set_size, id_size, payload_size, max_payload, trials, fake_data) = util::get_config_experiments(&parameters);

    for _i in 0..trials{
        run_server(set_size, id_size, max_payload, payload_size, fake_data);
    }

    println!("Experiments done !");
}
