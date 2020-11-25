use crate::{
    codecs::Codec, codecs::Options, utils::BytesToBytesDecoder, utils::BytesToBytesEncoder,
};

#[derive(Default)]
pub struct HexCodec;

impl Codec for HexCodec {
    fn run_codec(
        &self,
        mut input: &mut dyn std::io::Read,
        global_mode: crate::codecs::CodecMode,
        options: &Options,
        mut output: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        let use_capital = options.get_switch("c");

        match global_mode {
            crate::codecs::CodecMode::Encoding => {
                let mut capital_writer;
                let mut lower_writer;

                let mut writer = if use_capital {
                    capital_writer = BytesToBytesEncoder::new(&mut output, |buf| {
                        Ok((hex::encode_upper(buf).into_bytes(), &[]))
                    });
                    &mut capital_writer as &mut dyn std::io::Write
                } else {
                    lower_writer = BytesToBytesEncoder::new(&mut output, |buf| {
                        Ok((hex::encode(buf).into_bytes(), &[]))
                    });
                    &mut lower_writer as &mut dyn std::io::Write
                };

                std::io::copy(input, &mut writer)?;
                Ok(())
            }
            crate::codecs::CodecMode::Decoding => {
                let mut reader = BytesToBytesDecoder::new(
                    &mut input,
                    Box::new(|buf| {
                        let split_at = if buf.len() % 2 == 0 {
                            buf.len()
                        } else {
                            buf.len() - 1
                        };

                        let (input, remain) = buf.split_at(split_at);
                        let output = hex::decode(input).map_err(|err| {
                            std::io::Error::new(std::io::ErrorKind::InvalidData, err)
                        })?;

                        Ok((output, remain))
                    }),
                );

                std::io::copy(&mut reader, output)?;
                Ok(())
            }
        }
    }
}
