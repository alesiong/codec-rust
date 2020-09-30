use crate::codecs::{Codec, CodecUsage};

#[derive(Default)]
pub struct TeeCodecs;


impl Codec for TeeCodecs {
    fn run_codec(
        &self,
        input: &mut dyn std::io::Read,
        global_mode: crate::codecs::CodecMode,
        options: &std::collections::HashMap<String, String>,
        output: &mut dyn std::io::Write,
    ) -> Result<(), Box<dyn std::error::Error>> {

    }
}

#[derive(Default)]
pub struct SinkCodecs;
