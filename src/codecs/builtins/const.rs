use crate::codecs::{builtins::IdCodecs, Codec, CodecUsage, Options};

#[derive(Default)]
pub struct ConstCodecs(IdCodecs);

impl CodecUsage for ConstCodecs {
    fn usage(&self) -> String {
        "    -C replacement: ingore input, and replace the output with `replacement`\n".to_string()
    }
}

impl Codec for ConstCodecs {
    fn run_codec(
        &self,
        _input: &mut dyn std::io::Read,
        global_mode: crate::codecs::CodecMode,
        options: &Options,
        output: &mut dyn std::io::Write,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let value = options
            .get_text_raw("C")
            .ok_or(Box::<dyn std::error::Error>::from(
                "const: missing required option const value (-C)",
            ))?;

        self.0
            .run_codec(&mut value.as_slice(), global_mode, options, output)
    }
}
