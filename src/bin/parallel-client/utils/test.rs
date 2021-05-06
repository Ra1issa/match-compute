// Compute Aggregated Payloads associated with intersection
// in the clear for testing purposes.
use std::{
    convert::TryInto,
    collections::HashMap,
    path::PathBuf,
};
use match_compute::util;
use scuttlebutt::{Block512};

pub fn test(ids_client: &[Vec<u8>], ids_server: &[Vec<u8>],
                    payloads_client: &[Block512], payloads_server: &[Block512]) -> (u64, u64){


    let client_len = ids_client.len();
    let server_len = ids_server.len();

    let mut weighted_payload = 0;
    let mut sum_weights = 0;

    let mut sever_elements = HashMap::new();
    for i in 0..server_len{

        let id_server: &[u8] = &ids_server[i];
        let id_server: [u8; 8] = id_server.try_into().unwrap();
        let id_server = u64::from_le_bytes(id_server);

        let server_val = u64::from_le_bytes(payloads_server[i].prefix(8).try_into().unwrap());
        sever_elements.insert(
            id_server,
            server_val,
        );
    }

    for i in 0..client_len{

        let id_client: &[u8] = &ids_client[i];
        let id_client: [u8; 8] = id_client.try_into().unwrap();
        let id_client = u64::from_le_bytes(id_client);

        if sever_elements.contains_key(&id_client){
            let client_val = u64::from_le_bytes(payloads_client[i].prefix(8).try_into().unwrap());
            weighted_payload = weighted_payload + client_val*sever_elements.get(&id_client).unwrap();
            sum_weights = sum_weights + sever_elements.get(&id_client).unwrap();
            println!("server_val {:?}", ever_elements.get(&id_client));
        }
    }
    (weighted_payload, sum_weights)
}

pub fn clear_results(parameters: &HashMap<String, String>, path:&mut PathBuf,
                    ids_client: &[Vec<u8>], payloads_client: &[Block512],
                    precision: u32, fake_data: bool){
    let mut ids_server = Vec::new();
    let mut payloads_server = Vec::new();
    if fake_data == true {
        let (ids, payloads) = util::read_server_data(path);
        ids_server = ids;
        payloads_server = payloads;

    }else{
        let (_, server_path, _, schema_id, schema_payload) = util::get_config_sever(&parameters);
        let (ids, payloads) = util::parse_files(schema_id, schema_payload, &server_path);
        ids_server = ids;
        payloads_server = payloads;
    }

    let (aggregate, sum_weights) = test(&ids_client, &ids_server, &payloads_client, &payloads_server);

    let aggregate_adj: f64 = aggregate as f64/ 10_u64.pow(precision) as f64;
    let output: f64 = aggregate_adj / sum_weights as f64;

    println!("In the clear aggregate {:?}", aggregate_adj);
    println!("In the clear sum of weights {:?}", sum_weights);
    println!("In the clear average result {:?}", output);
}
