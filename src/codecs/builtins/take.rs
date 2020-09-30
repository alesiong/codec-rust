use std::io::Read;

use crate::codecs::{Codec, CodecUsage};

#[derive(Default)]
pub struct TakeCodecs;

impl Codec for TakeCodecs {
    fn run_codec(
        &self,
        input: &mut dyn std::io::Read,
        _global_mode: crate::codecs::CodecMode,
        options: &std::collections::HashMap<String, String>,
        output: &mut dyn std::io::Write,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut drop_bytes = 0;
        if let Some(t) = options.get("B") {
            drop_bytes = t.parse()?;
        };

        std::io::copy(&mut input.take(drop_bytes), output)?;

        Ok(())
    }
}
