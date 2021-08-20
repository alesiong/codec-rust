use std::io::Read;

use crate::codecs::{Codec, CodecUsage, Options};

#[derive(Default)]
pub struct DropCodecs;

impl Codec for DropCodecs {
    fn run_codec(
        &self,
        input: &mut dyn std::io::Read,
        _global_mode: crate::codecs::CodecMode,
        options: &Options,
        output: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        let drop_bytes = options.get_text("B")?.unwrap_or(0);

        std::io::copy(&mut input.take(drop_bytes), &mut std::io::sink())?;

        std::io::copy(input, output)?;

        Ok(())
    }
    fn as_codec_usage(&self) -> Option<&dyn CodecUsage> {
        Some(self)
    }
}

impl CodecUsage for DropCodecs {
    fn usage(&self) -> String {
        "    -B count: drop at most first `count` bytes from input\n".to_string()
    }
}
