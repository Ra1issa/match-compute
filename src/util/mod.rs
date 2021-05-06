use std::{
    env,
    fs::{File, read_to_string},
    io::{BufRead, BufReader, stdin, stdout, Read, Write},
    collections::HashMap,
    path::PathBuf,
};

use rand::{CryptoRng, Rng};
use fancy_garbling::{
     CrtBundle,
     Wire,
};
use scuttlebutt::{AesRng, Block512};
use serde_json;

pub fn int_vec_block512(values: Vec<u64>) -> Vec<Block512> {
    values.into_iter()
          .map(|item|{
            let value_bytes = item.to_le_bytes();
            let mut res_block = [0 as u8; 64];
            for i in 0..8{
                res_block[i] = value_bytes[i];
            }
            Block512::from(res_block)
         }).collect()
}
pub fn rand_u64_vec<RNG: CryptoRng + Rng>(n: usize, modulus: u64, rng: &mut RNG) -> Vec<u64>{
    (0..n).map(|_| rng.gen::<u64>()%modulus).collect()
}
pub fn enum_ids(n: usize, id_size: usize) ->Vec<Vec<u8>>{
    let mut ids = Vec::with_capacity(n);
    for i in 0..n as u64{
        let v:Vec<u8> = i.to_le_bytes().iter().take(id_size).cloned().collect();
        ids.push(v);
    }
    ids
}

pub fn generate_dummy_data(set_size: usize, id_size: usize, max_payload: u64)
    -> (Vec<Vec<u8>>, Vec<Block512>) {
    let mut rng = AesRng::new();
    let ids = enum_ids(set_size, id_size);
    let payloads = int_vec_block512(rand_u64_vec(set_size, max_payload,&mut rng));
    (ids, payloads)
}


pub fn crt_to_wires(v: &[CrtBundle<Wire>])-> Vec<Vec<Wire>>{
    v.into_iter()
     .map(|c| c.wires().to_vec()).collect()
}

pub fn wires_to_crt(v: &[Vec<Wire>])-> Vec<CrtBundle<Wire>>{
    v.into_iter()
     .map(|c| CrtBundle::new(c.to_vec())).collect()
}


pub fn pad_data<RNG: CryptoRng + Rng>(ids: &[Vec<u8>], payloads: &[Block512],
                        client_padding: usize, rng: &mut RNG) -> (Vec<Vec<u8>>, Vec<Block512>){

    let mut real_data = HashMap::new();
    for i in 0..ids.len(){
        let mut id = [0 as u8; 8];
        for j in 0..8{
            id[j] = ids[i][j];
        }
        real_data.insert(u64::from_le_bytes(id), payloads[i]);
    }
    let mut ids_padded = ids.to_vec();
    let mut payloads_padded = payloads.to_vec();

    for _i in 0..client_padding{
        let mut new_id = rng.gen::<u64>();
        while real_data.contains_key(&new_id){
            new_id = rng.gen::<u64>();
        }
        ids_padded.push(new_id.to_le_bytes().to_vec());
        payloads_padded.push(Block512::from([0 as u8; 64]));
    }
    (ids_padded, payloads_padded)
}

pub fn write_server_data(path:&mut PathBuf, ids: &[Vec<u8>], data: &[Block512]){
    path.pop();
    path.push("data");
    path.push("ids.txt");
    let path_str = path.clone().into_os_string().into_string().unwrap();
    let mut file_ids = File::create(path_str).unwrap();
    path.pop();

    let ids_json = serde_json::to_string(ids).unwrap();
    file_ids.write(&ids_json.as_bytes()).unwrap();

    path.push("payloads.txt");
    let path_str = path.clone().into_os_string().into_string().unwrap();
    let mut file_data = File::create(path_str).unwrap();
    path.pop();

    let data_json = serde_json::to_string(data).unwrap();
    file_data.write(&data_json.as_bytes()).unwrap();

    path.pop();
    path.push("src");
}

pub fn read_server_data(path:&mut PathBuf) -> (Vec<Vec<u8>>, Vec<Block512>){
    path.pop();
    path.pop();
    path.pop();
    path.push("data");

    path.push("ids.txt");
    let path_id = path.clone().into_os_string().into_string().unwrap();
    let ids: Vec<Vec<u8>> = serde_json::from_str(&read_to_string(path_id).unwrap()).unwrap();
    path.pop();

    path.push("payloads.txt");
    let path_data = path.clone().into_os_string().into_string().unwrap();
    let data: Vec<Block512> = serde_json::from_str(&read_to_string(path_data).unwrap()).unwrap();
    path.pop();

    path.pop();
    path.push("src");

    (ids, data)
}

pub fn get_path() -> PathBuf{
    let mut path = env::current_exe().unwrap();
    path.pop();
    path.pop();
    path.pop();
    path.push("src");
    path
}

/// Parse files for PSTY Payload computation.
pub fn parse_files(
    id_position: usize,
    payload_position: usize,
    path: &str,
) -> (Vec<Vec<u8>>, Vec<Block512>) {
    let data = File::open(path).unwrap();

    let buffer = BufReader::new(data).lines();

    let mut ids = Vec::new();
    let mut payloads = Vec::new();

    let mut cnt = 0;
    for line in buffer.enumerate() {
        let line_split = line
            .1
            .unwrap()
            .split(',')
            .map(|item| item.to_string())
            .collect::<Vec<String>>();
        if cnt == 0 {
            cnt += 1;
        } else {
            ids.push(
                line_split[id_position]
                    .parse::<u64>()
                    .unwrap()
                    .to_le_bytes()
                    .to_vec(),
            );
            payloads.push(line_split[payload_position].parse::<u64>().unwrap());
        }
    }
    (ids, int_vec_block512(payloads))
}

pub fn parse_config(path_config: &mut PathBuf) -> HashMap<String, String>{
    path_config.push("config/configuration.txt");
    let absolute_path = path_config.clone().into_os_string().into_string().unwrap();
    let configuration = File::open(absolute_path).unwrap();
    let buffer = BufReader::new(configuration).lines();
    let mut parameters = HashMap::new();
    for line in buffer.enumerate(){
        let read_line =  line.1.unwrap();
        if !read_line.is_empty(){
            let line_split = read_line.split(": ").map(|item| item.to_string()).collect::<Vec<String>>();
            parameters.insert(line_split[0].clone(), line_split[1].clone());
        }
    }
    parameters
}

pub fn get_config_sever(parameters: &HashMap<String, String>)->
                                    (String, String, usize, usize, usize){
    let address = parameters.get("address").unwrap().to_owned();
    let server_path = parameters.get("data_path_server").unwrap().to_owned();
    let nthread = parameters.get("nthread").unwrap().parse::<usize>().unwrap();
    //
    let id_position = parameters.get("id_position_server").unwrap().parse::<usize>().unwrap();
    let payload_position = parameters.get("payload_position_server").unwrap().parse::<usize>().unwrap();

    (address, server_path, nthread, id_position, payload_position)
}

pub fn get_config_client(parameters: &HashMap<String, String>)->
            (String, String, u64, u32, usize, usize, usize, usize, usize){
    let address = parameters.get("address").unwrap().to_owned();
    let client_path = parameters.get("data_path_client").unwrap().to_owned();

    let sleeptime = parameters.get("sleeptime").unwrap().parse::<u64>().unwrap();
    let precision = parameters.get("precision").unwrap().parse::<u32>().unwrap();

    let nthread = parameters.get("nthread").unwrap().parse::<usize>().unwrap();
    let megasize = parameters.get("megasize").unwrap().parse::<usize>().unwrap();

    let id_position = parameters.get("id_position_client").unwrap().parse::<usize>().unwrap();
    let payload_position = parameters.get("payload_position_client").unwrap().parse::<usize>().unwrap();
    let client_padding = parameters.get("client_padding").unwrap().parse::<usize>().unwrap();

    (address, client_path, sleeptime, precision, nthread, megasize, client_padding, id_position, payload_position)
}

// Taken from:
// https://www.reddit.com/r/rust/comments/8tfyof/noob_question_pause/e177530?utm_source=share&utm_medium=web2x&context=3
fn _pause() {
    let mut stdout = stdout();
    stdout.write(b"Press Enter to continue...").unwrap();
    stdout.flush().unwrap();
    stdin().read(&mut [0]).unwrap();
}

fn _windows_hang_executable(){
    std::panic::set_hook(Box::new(|p| {
        println!("{}", p);
        loop { }
    }));
}
