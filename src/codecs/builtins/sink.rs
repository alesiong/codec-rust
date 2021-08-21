use crate::{
    codecs::Codec,
    codecs::{CodecUsage, Options},
    utils::MultiWriter,
};

#[derive(Default)]
pub struct TeeCodecs;

impl Codec for TeeCodecs {
    fn run_codec(
        &self,
        input: &mut dyn std::io::Read,
        _global_mode: crate::codecs::CodecMode,
        options: &Options,
        output: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        let mut writers = Vec::with_capacity(2);

        let mut file;

        if let Some(output_file) = options.get_text::<String>("O")? {
            file = std::fs::File::create(output_file)?;
            writers.push(&mut file as &mut dyn std::io::Write);
        }

        if !options.get_switch("c") {
            writers.push(output);
        }

        let mut writer = MultiWriter::new(writers);
        std::io::copy(input, &mut writer)?;

        Ok(())
    }
    fn as_codec_usage(&self) -> Option<&dyn CodecUsage> {
        Some(self)
    }
}

impl CodecUsage for TeeCodecs {
    fn usage(&self) -> String {
        "    (if with no argument, behave like `id`)
    -c: (close output) do not write to output
    -O file: also write to `file`, optional
"
        .to_string()
    }
}

#[derive(Default)]
pub struct SinkCodecs(TeeCodecs);

impl Codec for SinkCodecs {
    fn run_codec(
        &self,
        input: &mut dyn std::io::Read,
        global_mode: crate::codecs::CodecMode,
        _options: &Options,
        output: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        let mut options = Options::new();
        options.insert_switch("c");

        self.0.run_codec(input, global_mode, &options, output)
    }
    fn as_codec_usage(&self) -> Option<&dyn CodecUsage> {
        Some(self)
    }
}

impl CodecUsage for SinkCodecs {
    fn usage(&self) -> String {
        "    (= tee -c or redirect -O /dev/null on unix-like systems)
    differences with repeat: repeat without arguments (=repeat -T 0) will end the
    execution of the whole chain immediately, e.g.:
    const -C example tee -O /dev/stdout sink
        will output example
    const -C example tee -O /dev/stdout repeat
        will output nothing
"
        .to_string()
    }
}

#[derive(Default)]
pub struct RedirectCodecs(TeeCodecs);

impl Codec for RedirectCodecs {
    fn run_codec(
        &self,
        input: &mut dyn std::io::Read,
        global_mode: crate::codecs::CodecMode,
        options: &Options,
        output: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        let output_file = options.get_text_raw("O").ok_or_else(|| {
            anyhow::anyhow!("redirect: missing required option output file (-O)",)
        })?;
        let mut options = Options::new();
        options.insert_switch("c");
        options.insert_text("O", output_file);

        self.0.run_codec(input, global_mode, &options, output)
    }
    fn as_codec_usage(&self) -> Option<&dyn CodecUsage> {
        Some(self)
    }
}

impl CodecUsage for RedirectCodecs {
    fn usage(&self) -> String {
        "    = tee -c -O `file`
    -O file: redirect output to `file`
"
        .to_string()
    }
}
