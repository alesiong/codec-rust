pub mod builtins;
pub mod meta;

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
        options: &Options,
        output: &mut dyn Write,
    ) -> anyhow::Result<()>;

    fn as_codec_usage(&self) -> Option<&dyn CodecUsage> {
        None
    }
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

pub struct Options {
    options: HashMap<String, Option<Vec<u8>>>,
}

impl Options {
    pub fn new() -> Self {
        Options {
            options: HashMap::new(),
        }
    }

    pub fn insert_switch(&mut self, name: &str) {
        debug_assert!(name.chars().next().unwrap().is_lowercase());
        self.options.insert(name.to_string(), None);
    }

    pub fn insert_text(&mut self, name: &str, value: &[u8]) {
        debug_assert!(name.chars().next().unwrap().is_uppercase());
        self.options.insert(name.to_string(), Some(value.to_vec()));
    }

    pub fn insert_text_str<T: ToString + Copy>(&mut self, name: &str, value: T) {
        self.insert_text(name, value.to_string().as_bytes());
    }

    pub fn get_switch(&self, name: &str) -> bool {
        debug_assert!(name.chars().next().unwrap().is_lowercase());
        self.options.contains_key(name)
    }

    pub fn get_text_raw(&self, name: &str) -> Option<Vec<u8>> {
        debug_assert!(name.chars().next().unwrap().is_uppercase());
        Some(self.options.get(name)?.clone().unwrap())
    }

    pub fn get_text<T>(&self, name: &str) -> anyhow::Result<Option<T>>
    where
        T: std::str::FromStr,
        <T as std::str::FromStr>::Err: 'static + Error + Sync + Send,
    {
        debug_assert!(name.chars().next().unwrap().is_uppercase());

        let text = match self.get_text_raw(name) {
            Some(text) => text,
            None => return Ok(None),
        };

        Ok(Some(String::from_utf8(text)?.parse()?))
    }
}

pub trait MetaCodec {
    fn run_meta_codec(
        &self,
        input: &mut dyn Read,
        global_mode: CodecMode,
        options: &Options,
        codec_meta_info: &CodecMetaInfo,
        output: &mut dyn Write,
    ) -> anyhow::Result<()>;
}

struct MetaCodecWrapper<'a, T> {
    meta: T,
    meta_info: &'a CodecMetaInfo,
}

impl<'a, T: MetaCodec> Codec for MetaCodecWrapper<'a, T> {
    fn run_codec(
        &self,
        input: &mut dyn Read,
        global_mode: CodecMode,
        options: &Options,
        output: &mut dyn Write,
    ) -> anyhow::Result<()> {
        self.meta
            .run_meta_codec(input, global_mode, options, self.meta_info, output)
    }
}
