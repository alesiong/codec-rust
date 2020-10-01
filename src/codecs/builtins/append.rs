use crate::codecs::Codec;

#[derive(Default)]
pub struct AppendCodecs;

#[derive(Default)]
pub struct NewLineCodecs(AppendCodecs);

impl Codec for AppendCodecs {
    fn run_codec(
        &self,
        input: &mut dyn std::io::Read,
        _global_mode: crate::codecs::CodecMode,
        options: &std::collections::HashMap<String, String>,
        output: &mut dyn std::io::Write,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let value = options.get("A").ok_or(Box::<dyn std::error::Error>::from(
            "append: missing required option append value (-A)",
        ))?;

        let _ = std::io::copy(input, output)?;
        output.write_all(value.as_bytes())?;
        Ok(())
    }
}

impl Codec for NewLineCodecs {
    fn run_codec(
        &self,
        input: &mut dyn std::io::Read,
        global_mode: crate::codecs::CodecMode,
        _options: &std::collections::HashMap<String, String>,
        output: &mut dyn std::io::Write,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut options = std::collections::HashMap::new();
        options.insert("A".to_string(), "\n".to_string());

        self.0.run_codec(input, global_mode, &options, output)
    }
}
