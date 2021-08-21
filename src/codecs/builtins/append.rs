use crate::codecs::{Codec, CodecUsage, Options};

#[derive(Default)]
pub struct AppendCodecs;

#[derive(Default)]
pub struct NewLineCodecs(AppendCodecs);

impl Codec for AppendCodecs {
    fn run_codec(
        &self,
        input: &mut dyn std::io::Read,
        _global_mode: crate::codecs::CodecMode,
        options: &Options,
        output: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        let value = options
            .get_text_raw("A")
            .ok_or_else(|| anyhow::anyhow!("append: missing required option append value (-A)",))?;

        let _ = std::io::copy(input, output)?;
        output.write_all(value)?;
        Ok(())
    }
    fn as_codec_usage(&self) -> Option<&dyn CodecUsage> {
        Some(self)
    }
}

impl CodecUsage for AppendCodecs {
    fn usage(&self) -> String {
        "    -A string: pass input to output, and then append `string`\n".to_string()
    }
}

impl Codec for NewLineCodecs {
    fn run_codec(
        &self,
        input: &mut dyn std::io::Read,
        global_mode: crate::codecs::CodecMode,
        _options: &Options,
        output: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        let mut options = Options::new();
        options.insert_text("A", b"\n");

        self.0.run_codec(input, global_mode, &options, output)
    }
    fn as_codec_usage(&self) -> Option<&dyn CodecUsage> {
        Some(self)
    }
}

impl CodecUsage for NewLineCodecs {
    fn usage(&self) -> String {
        "    (= append -A ['\\n' escape -d])
    append new line
"
        .to_string()
    }
}
