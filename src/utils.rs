use std::{
    env,
    fs::{File},
    io::{BufRead, BufReader, stdin, stdout, Read, Write},
    collections::HashMap,
    time::{Duration},
    time::SystemTime,
    thread,
};


pub fn parse_config() -> HashMap<String, String>{
    let mut path = env::current_exe().unwrap();
    path.pop();
    path.pop();
    path.push("match-compute-parallel");
    path.push("configuration.txt");
    let absolute_path = path.clone().into_os_string().into_string().unwrap();
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
