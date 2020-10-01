use crate::{codecs::Codec, utils::MultiWriter};

#[derive(Default)]
pub struct TeeCodecs;

impl Codec for TeeCodecs {
    fn run_codec(
        &self,
        input: &mut dyn std::io::Read,
        _global_mode: crate::codecs::CodecMode,
        options: &std::collections::HashMap<String, String>,
        output: &mut dyn std::io::Write,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut writers = Vec::with_capacity(2);

        let mut file;

        if let Some(output_file) = options.get("O") {
            file = std::fs::File::create(output_file)?;
            writers.push(&mut file as &mut dyn std::io::Write);
        }

        if !options.contains_key("c") {
            writers.push(output);
        }

        let mut writer = MultiWriter::new(writers);
        std::io::copy(input, &mut writer)?;

        Ok(())
    }
}

#[derive(Default)]
pub struct SinkCodecs(TeeCodecs);

impl Codec for SinkCodecs {
    fn run_codec(
        &self,
        input: &mut dyn std::io::Read,
        global_mode: crate::codecs::CodecMode,
        _options: &std::collections::HashMap<String, String>,
        output: &mut dyn std::io::Write,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut options = std::collections::HashMap::new();
        options.insert("c".to_string(), "*".to_string());

        self.0.run_codec(input, global_mode, &options, output)
    }
}

#[derive(Default)]
pub struct RedirectCodecs(TeeCodecs);

impl Codec for RedirectCodecs {
    fn run_codec(
        &self,
        input: &mut dyn std::io::Read,
        global_mode: crate::codecs::CodecMode,
        options: &std::collections::HashMap<String, String>,
        output: &mut dyn std::io::Write,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let output_file = options.get("O").ok_or(Box::<dyn std::error::Error>::from(
            "redirect: missing required option output file (-O)",
        ))?;

        let mut options = std::collections::HashMap::new();
        options.insert("c".to_string(), "*".to_string());
        options.insert("O".to_string(), output_file.clone());

        self.0.run_codec(input, global_mode, &options, output)
    }
}
