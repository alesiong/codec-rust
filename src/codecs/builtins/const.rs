use crate::codecs::{builtins::IdCodecs, Codec, CodecUsage};

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
        options: &std::collections::HashMap<String, String>,
        output: &mut dyn std::io::Write,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let value = options.get("C").ok_or(Box::<dyn std::error::Error>::from(
            "const: missing required option const value (-C)",
        ))?;

        self.0.run_codec(
            &mut value.as_bytes(),
            global_mode,
            options,
            output,
        )
    }
}
