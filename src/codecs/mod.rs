pub mod builtins;
pub mod meta;

use std::{
    collections::{btree_map::Iter, BTreeMap, HashMap},
    error::Error,
    io::{Read, Write},
    ops::Deref,
};

use once_cell::sync::OnceCell;

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
    codecs_map: BTreeMap<String, Box<dyn Codec + Send + Sync>>,
}

impl CodecMetaInfo {
    pub fn new() -> CodecMetaInfo {
        CodecMetaInfo {
            codecs_map: BTreeMap::new(),
        }
    }

    pub fn instance() -> &'static CodecMetaInfo {
        GLOBAL_CODEC_META_INFO
            .get()
            .expect("GLOBAL_CODEC_META_INFO should have been initialized")
    }

    pub fn set_instance(self) {
        GLOBAL_CODEC_META_INFO
            .set(self)
            .ok()
            .expect("GLOBAL_CODEC_META_INFO initialized failed");
    }

    pub fn register(&mut self, name: &str, codec: Box<dyn Codec + Send + Sync>) {
        self.codecs_map.insert(name.to_string(), codec);
    }

    pub fn register_meta<T: 'static + MetaCodec + Send + Sync>(&mut self, name: &str, meta: T) {
        self.register(name, Box::new(MetaCodecWrapper { meta }));
    }

    pub fn register_codec<C: 'static + Codec + Default + Send + Sync>(&mut self, name: &str) {
        self.register(name, Box::<C>::default());
    }

    pub fn lookup(&self, name: &str) -> Option<&(dyn Codec + Send + Sync)> {
        self.codecs_map.get(name).map(|v| v.deref())
    }

    pub fn codecs_iter(&self) -> Iter<'_, String, Box<dyn Codec + Send + Sync>> {
        self.codecs_map.iter()
    }
}

static GLOBAL_CODEC_META_INFO: OnceCell<CodecMetaInfo> = OnceCell::new();

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

struct MetaCodecWrapper<T: ?Sized> {
    meta: T,
}

impl<'a, 'i: 'a, T: MetaCodec + ?Sized> Codec for MetaCodecWrapper<T> {
    fn run_codec(
        &self,
        input: &mut dyn Read,
        global_mode: CodecMode,
        options: &Options,
        output: &mut dyn Write,
    ) -> anyhow::Result<()> {
        self.meta.run_meta_codec(
            input,
            global_mode,
            options,
            CodecMetaInfo::instance(),
            output,
        )
    }
}
