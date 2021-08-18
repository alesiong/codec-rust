use crate::{
    codecs::{Codec, CodecUsage, Options},
    utils::MultiWriter,
};

#[derive(Default)]

pub struct RepeatCodecs;

#[derive(Default)]
pub struct IdCodecs(RepeatCodecs);

impl CodecUsage for RepeatCodecs {
    fn usage(&self) -> String {
        "    -T times: repeat input for `times` times (int, >=0, default 0)".to_string()
    }
}

impl Codec for RepeatCodecs {
    fn run_codec(
        &self,
        input: &mut dyn std::io::Read,
        _global_mode: crate::codecs::CodecMode,
        options: &Options,
        output: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        let times = options.get_text("T")?.unwrap_or(0);
        if times < 0 {
            anyhow::bail!("repeat: times cannot be minus");
        }

        // TODO: (prof) consider new with capacity
        let mut buffer = Vec::<u8>::with_capacity(1024 * 8);

        if times > 0 {
            let mut writer = MultiWriter::new(vec![output, &mut buffer]);
            let _ = std::io::copy(input, &mut writer)?;

            for _ in 1..times {
                let _ = output.write_all(&buffer)?;
            }
        }

        Ok(())
    }

    fn as_codec_usage(&self) -> Option<&dyn CodecUsage> {
        Some(self as &dyn CodecUsage)
    }
}

impl CodecUsage for IdCodecs {
    fn usage(&self) -> String {
        "    pass input to output as is".to_string()
    }
}

impl Codec for IdCodecs {
    fn run_codec(
        &self,
        input: &mut dyn std::io::Read,
        global_mode: crate::codecs::CodecMode,
        _options: &Options,
        output: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        let mut options = Options::new();
        options.insert_text_str("T", 1);

        self.0.run_codec(input, global_mode, &options, output)
    }

    fn as_codec_usage(&self) -> Option<&dyn CodecUsage> {
        Some(self as &dyn CodecUsage)
    }
}
