use crate::{codecs::Codec, utils::BytesToBytesDecoder, utils::BytesToBytesEncoder};

#[derive(Default)]
pub struct HexCodec;

impl Codec for HexCodec {
    fn run_codec(
        &self,
        mut input: &mut dyn std::io::Read,
        global_mode: crate::codecs::CodecMode,
        options: &std::collections::HashMap<String, String>,
        mut output: &mut dyn std::io::Write,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let use_capital = options.contains_key("c");

        match global_mode {
            crate::codecs::CodecMode::Encoding => {
                let mut writer = if use_capital {
                    Box::new(BytesToBytesEncoder::new(&mut output, |buf| {
                        hex::encode_upper(buf).into_bytes()
                    })) as Box<dyn std::io::Write>
                } else {
                    Box::new(BytesToBytesEncoder::new(&mut output, |buf| {
                        hex::encode(buf).into_bytes()
                    })) as Box<dyn std::io::Write>
                };

                std::io::copy(input, &mut writer)?;
                Ok(())
            }
            crate::codecs::CodecMode::Decoding => {
                let mut reader = BytesToBytesDecoder::new(&mut input, |buf| {
                    let split_at = if buf.len() % 2 == 0 {
                        buf.len()
                    } else {
                        buf.len() - 1
                    };

                    let (input, remain) = buf.split_at(split_at);
                    let output = hex::decode(input)
                        .map_err(|err| std::io::Error::new(std::io::ErrorKind::InvalidData, err))?;

                    Ok((output, remain))
                });

                std::io::copy(&mut reader, output)?;
                Ok(())
            }
        }
    }
}
