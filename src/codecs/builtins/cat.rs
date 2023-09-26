use crate::codecs::{Codec, CodecUsage, Options};

#[derive(Default)]
pub struct CatCodecs;

impl Codec for CatCodecs {
    fn run_codec(
        &self,
        input: &mut dyn std::io::Read,
        _global_mode: crate::codecs::CodecMode,
        options: &Options,
        output: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        if !options.get_switch("c") {
            let _ = std::io::copy(input, output)?;
        }

        options
            .get_text::<String>("F")?
            .map(std::fs::File::open)
            .transpose()?
            .map(|mut file| std::io::copy(&mut file, output))
            .transpose()?;

        Ok(())
    }
    fn as_codec_usage(&self) -> Option<&dyn CodecUsage> {
        Some(self)
    }
}

impl CodecUsage for CatCodecs {
    fn usage(&self) -> String {
        "    (if with no argument, behave like `id`)
    -c: (close input) do not read from input
    -F file: also read from `file`, optional
"
            .to_string()
    }
}
