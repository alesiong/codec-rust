use std::io::Read;

use crate::codecs::{Codec, CodecUsage, Options};

#[derive(Default)]
pub struct TakeCodecs;

impl Codec for TakeCodecs {
    fn run_codec(
        &self,
        input: &mut dyn std::io::Read,
        _global_mode: crate::codecs::CodecMode,
        options: &Options,
        output: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        let drop_bytes: u64 = options.get_text("B")?.unwrap_or(0);

        std::io::copy(&mut input.take(drop_bytes), output)?;

        Ok(())
    }
    fn as_codec_usage(&self) -> Option<&dyn CodecUsage> {
        Some(self)
    }
}

impl CodecUsage for TakeCodecs {
    fn usage(&self) -> String {
        "    -B count: take up to first `count` bytes from input\n".to_string()
    }
}
