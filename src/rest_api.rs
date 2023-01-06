use std::collections::HashMap;
use std::fmt::Error;
use std::str::FromStr;

use rand::seq::SliceRandom;
use reqwest::blocking::{Body, Response};
use reqwest::header::{ACCEPT, ACCEPT_ENCODING, CONNECTION, CONTENT_LENGTH, CONTENT_TYPE, DNT, HeaderMap, HeaderName, HeaderValue, USER_AGENT};
use reqwest::Proxy;
use serde_json::{Map, Value};

use crate::ProxyType;

const USER_AGENTS: [&str; 20] = [
    "Mozilla/5.0 (X11; Linux x86_64; rv:103.0) Gecko/20100101 Firefox/103.0",
    "Mozilla/5.0 (X11; Linux x86_64; rv:103.0) Gecko/20100101 Firefox/105.0",
    "Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/537.1 (KHTML, like Gecko) Chrome/22.0.1207.1 Safari/537.1",
    "Mozilla/5.0 (X11; CrOS i686 2268.111.0) AppleWebKit/536.11 (KHTML, like Gecko) Chrome/20.0.1132.57 Safari/536.11",
    "Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/536.6 (KHTML, like Gecko) Chrome/20.0.1092.0 Safari/536.6",
    "Mozilla/5.0 (Windows NT 6.2) AppleWebKit/536.6 (KHTML, like Gecko) Chrome/20.0.1090.0 Safari/536.6",
    "Mozilla/5.0 (Windows NT 6.2; WOW64) AppleWebKit/537.1 (KHTML, like Gecko) Chrome/19.77.34.5 Safari/537.1",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/536.5 (KHTML, like Gecko) Chrome/19.0.1084.9 Safari/536.5",
    "Mozilla/5.0 (Windows NT 6.0) AppleWebKit/536.5 (KHTML, like Gecko) Chrome/19.0.1084.36 Safari/536.5",
    "Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/536.3 (KHTML, like Gecko) Chrome/19.0.1063.0 Safari/536.3",
    "Mozilla/5.0 (Windows NT 5.1) AppleWebKit/536.3 (KHTML, like Gecko) Chrome/19.0.1063.0 Safari/536.3",
    "Mozilla/5.0 (Macintosh; Intel Mac OS X 10_8_0) AppleWebKit/536.3 (KHTML, like Gecko) Chrome/19.0.1063.0 Safari/536.3",
    "Mozilla/5.0 (Windows NT 6.2) AppleWebKit/536.3 (KHTML, like Gecko) Chrome/19.0.1062.0 Safari/536.3",
    "Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/536.3 (KHTML, like Gecko) Chrome/19.0.1062.0 Safari/536.3",
    "Mozilla/5.0 (Windows NT 6.2) AppleWebKit/536.3 (KHTML, like Gecko) Chrome/19.0.1061.1 Safari/536.3",
    "Mozilla/5.0 (Windows NT 6.1; WOW64) AppleWebKit/536.3 (KHTML, like Gecko) Chrome/19.0.1061.1 Safari/536.3",
    "Mozilla/5.0 (Windows NT 6.1) AppleWebKit/536.3 (KHTML, like Gecko) Chrome/19.0.1061.1 Safari/536.3",
    "Mozilla/5.0 (Windows NT 6.2) AppleWebKit/536.3 (KHTML, like Gecko) Chrome/19.0.1061.0 Safari/536.3",
    "Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/535.24 (KHTML, like Gecko) Chrome/19.0.1055.1 Safari/535.24",
    "Mozilla/5.0 (Windows NT 6.2; WOW64) AppleWebKit/535.24 (KHTML, like Gecko) Chrome/19.0.1055.1 Safari/535.24"
];

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RequestType {
    GET,
    POST,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RestAPI {
    pub url: String,
    pub header: Value,
    pub data: String,
    pub request_type: RequestType,
}

impl RestAPI {
    pub fn from_json(json: &Map<String, Value>) -> Result<Self, Error> {
        let url = json.get("url").unwrap_or(&Value::Null).as_str().expect("API URL is not a string").to_string();
        let request_type = match json.get("method").unwrap_or(&Value::String(String::from("POST"))).as_str().expect("API method is not valid") {
            "GET" => RequestType::GET,
            "POST" => RequestType::POST,
            _ => panic!("Invalid request type, must be GET or POST")
        };
        let header = json.get("header").unwrap_or(&Value::Null).clone();
        let data = json.get("data").unwrap_or(&Value::String(String::from(""))).as_str().unwrap_or("").to_string();
        Ok(Self {
            url,
            header,
            data,
            request_type,
        })
    }

    pub fn request(&self, proxy_type: Option<ProxyType>, proxy: &str) -> reqwest::Result<Response> {
        let header_map = self.create_header_map();
        let mut client = reqwest::blocking::Client::builder()
            .default_headers(header_map.clone())
            .cookie_store(true);

        if !proxy.is_empty() && proxy_type.is_some() {
            match proxy_type.unwrap() {
                ProxyType::HTTP => client = client.proxy(Proxy::http(proxy).unwrap()),
                ProxyType::HTTPS => client = client.proxy(Proxy::https(proxy).unwrap()),
                ProxyType::SOCKS5 => client = client.proxy(Proxy::all(proxy).unwrap())
            }
        }

        let client = client.build().unwrap();

        let request_builder = match self.request_type {
            RequestType::GET => client.get(&self.url),
            RequestType::POST => client.post(&self.url)
        };

        let request = request_builder.headers(header_map)
            .body(Body::from(self.data.clone()))
            .build();
        client.execute(request?)
    }

    fn create_header_map(&self) -> HeaderMap {
        let mut header_map = HeaderMap::new();
        header_map.insert(ACCEPT, HeaderValue::from_str("application/json, text/plain, */*").unwrap());
        header_map.insert(ACCEPT_ENCODING, HeaderValue::from_str("gzip, deflate, br").unwrap());
        header_map.insert(CONNECTION, HeaderValue::from_str("keep-alive").unwrap());
        header_map.insert(CONTENT_TYPE, HeaderValue::from_str("application/json").unwrap());
        header_map.insert(HeaderName::from_str("Sec-Fetch-Dest").unwrap(), HeaderValue::from_str("empty").unwrap());
        header_map.insert(HeaderName::from_str("Sec-Fetch-Mode").unwrap(), HeaderValue::from_str("cors").unwrap());
        header_map.insert(HeaderName::from_str("Sec-Fetch-Site").unwrap(), HeaderValue::from_str("same-site").unwrap());
        header_map.insert(HeaderName::from_str("sec-ch-ua").unwrap(), HeaderValue::from_str("\"Opera GX\";v=\"93\", \"Not/A)Brand\";v=\"8\", \"Chromium\";v=\"107\"").unwrap());
        header_map.insert(HeaderName::from_str("sec-ch-ua-mobile").unwrap(), HeaderValue::from_str("?0").unwrap());
        header_map.insert(HeaderName::from_str("sec-ch-ua-platform").unwrap(), HeaderValue::from_str("\"Windows\"").unwrap());
        header_map.insert(DNT, HeaderValue::from_str("1").unwrap());
        header_map.insert(CONTENT_LENGTH, HeaderValue::from_str(&self.data.len().to_string()).unwrap());
        let user_agent = USER_AGENTS.choose(&mut rand::thread_rng()).unwrap().clone();
        header_map.insert(USER_AGENT, HeaderValue::from_str(&user_agent).unwrap());
        header_map.insert(HeaderName::from_str("X-User-Agent").unwrap(), HeaderValue::from_str(&format!("{user_agent} FKUA/website/42/website/Desktop")).unwrap());
        match &self.header {
            Value::Object(map) => {
                for (key, value) in map.iter() {
                    if !value.is_string() { continue; }
                    header_map.insert(HeaderName::from_str(key).unwrap(), HeaderValue::from_str(value.as_str().unwrap()).unwrap());
                }
            }
            _ => {}
        }
        header_map
    }

    pub fn format(&self, replaces: &HashMap<String, String>) -> Self {
        Self {
            url: self.formatter(self.url.clone(), replaces),
            header: self.json_formatter(self.header.clone(), replaces),
            data: self.formatter(self.data.clone(), replaces),
            request_type: self.request_type.clone(),
        }
    }

    fn formatter(&self, mut str: String, replaces: &HashMap<String, String>) -> String {
        for (key, value) in replaces {
            str = str.replace(&format!("<{key}>"), value);
        }
        str
    }

    fn json_formatter(&self, json: Value, replaces: &HashMap<String, String>) -> Value {
        match json {
            Value::Null => Value::Null,
            Value::String(s) => Value::String(self.formatter(s, replaces)),
            Value::Number(n) => Value::Number(n),
            Value::Bool(b) => Value::Bool(b),
            Value::Object(obj) => {
                let mut new_json: Map<String, Value> = Map::new();
                for (key, value) in obj.iter() {
                    new_json.insert(self.formatter(key.clone(), replaces), self.json_formatter(value.clone(), replaces));
                }
                Value::Object(new_json)
            }
            Value::Array(arr) => {
                let mut vector: Vec<Value> = Vec::new();
                for value in arr.iter() {
                    vector.push(self.json_formatter(value.clone(), replaces));
                }
                Value::Array(vector)
            }
        }
    }
}