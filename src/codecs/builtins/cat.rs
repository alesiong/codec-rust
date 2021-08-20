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
        let input_file = options
            .get_text::<String>("F")?
            .ok_or_else(|| anyhow::anyhow!("cat: missing required option input file (-F)"))?;

        if !options.get_switch("c") {
            let _ = std::io::copy(input, output)?;
        }

        let mut file = std::fs::File::open(input_file)?;
        let _ = std::io::copy(&mut file, output)?;

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
