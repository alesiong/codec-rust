use flate2::Compression;

use crate::codecs::{Codec, CodecMode, Options};

#[derive(Default)]
pub struct ZlibCodec;

impl Codec for ZlibCodec {
    fn run_codec(
        &self,
        input: &mut dyn std::io::Read,
        global_mode: CodecMode,
        options: &Options,
        output: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        match global_mode {
            CodecMode::Encoding => {
                let level = options
                    .get_text("L")?
                    .unwrap_or_else(|| Compression::default().level());

                let mut writer = flate2::write::ZlibEncoder::new(output, Compression::new(level));

                std::io::copy(input, &mut writer)?;

                Ok(())
            }
            CodecMode::Decoding => {
                let mut reader = flate2::read::ZlibDecoder::new(input);

                std::io::copy(&mut reader, output)?;

                Ok(())
            }
        }
    }
}
