pub mod builtins;

use std::{
    collections::HashMap,
    error::Error,
    io::{Read, Write},
    ops::Deref,
};

#[derive(Copy, Clone)]
pub enum CodecMode {
    Encoding,
    Decoding,
}

pub trait Codec {
    fn run_codec(
        &self,
        input: &mut dyn Read,
        global_mode: CodecMode,
        options: &HashMap<String, String>,
        output: &mut dyn Write,
    ) -> Result<(), Box<dyn Error>>;
}

pub trait CodecUsage {
    fn usage(&self) -> String;
}

pub struct CodecMetaInfo {
    codecs_map: HashMap<String, Box<dyn Codec + Send + Sync>>,
}

impl CodecMetaInfo {
    pub fn new() -> CodecMetaInfo {
        CodecMetaInfo {
            codecs_map: HashMap::new(),
        }
    }

    pub fn register(&mut self, name: &str, codec: Box<dyn Codec + Send + Sync>) {
        self.codecs_map.insert(name.to_string(), codec);
    }

    pub fn register_codec<C: 'static + Codec + Default + Send + Sync>(&mut self, name: &str) {
        self.register(name, Box::<C>::default());
    }

    pub fn lookup(&self, name: &str) -> Option<&(dyn Codec + Send + Sync)> {
        self.codecs_map.get(name).map(|v| v.deref())
    }
}
