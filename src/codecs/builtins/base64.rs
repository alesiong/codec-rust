use crate::codecs::{Codec, CodecUsage, Options};

#[derive(Default)]
pub struct Base64Codec;

impl Codec for Base64Codec {
    fn run_codec(
        &self,
        mut input: &mut dyn std::io::Read,
        global_mode: crate::codecs::CodecMode,
        options: &Options,
        mut output: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        let no_padding = options.get_switch("p");
        #[allow(clippy::collapsible_else_if)]
            let encoding = &if options.get_switch("u") {
            if no_padding {
                base64::engine::general_purpose::URL_SAFE_NO_PAD
            } else {
                base64::engine::general_purpose::URL_SAFE
            }
        } else {
            if no_padding {
                base64::engine::general_purpose::STANDARD_NO_PAD
            } else {
                base64::engine::general_purpose::STANDARD
            }
        };
        #[warn(clippy::collapsible_else_if)]

        match global_mode {
            crate::codecs::CodecMode::Encoding => {
                let mut encoder = base64::write::EncoderWriter::new(&mut output, encoding);
                std::io::copy(input, &mut encoder)?;
                Ok(())
            }
            crate::codecs::CodecMode::Decoding => {
                let mut decoder = base64::read::DecoderReader::new(&mut input, encoding);
                std::io::copy(&mut decoder, output)?;
                Ok(())
            }
        }
    }

    fn as_codec_usage(&self) -> Option<&dyn CodecUsage> {
        Some(self)
    }
}

impl CodecUsage for Base64Codec {
    fn usage(&self) -> String {
        "    -u: use url base64 instead\n".to_string()
    }
}
