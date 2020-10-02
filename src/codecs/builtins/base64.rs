use crate::codecs::{Codec, Options};

#[derive(Default)]
pub struct Base64Codec;

impl Codec for Base64Codec {
    fn run_codec(
        &self,
        mut input: &mut dyn std::io::Read,
        global_mode: crate::codecs::CodecMode,
        options: &Options,
        mut output: &mut dyn std::io::Write,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let no_padding = options.get_switch("p");
        let encoding = if options.get_switch("u") {
            if no_padding {
                base64::URL_SAFE_NO_PAD
            } else {
                base64::URL_SAFE
            }
        } else {
            if no_padding {
                base64::STANDARD_NO_PAD
            } else {
                base64::STANDARD
            }
        };

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
}
