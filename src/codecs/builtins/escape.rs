use snailquote::UnescapeError;

use crate::{
    codecs::{Codec, CodecMode, Options},
    utils::{BytesToBytesDecoder, BytesToBytesEncoder},
};

#[derive(Default)]
pub struct EscapeCodec;

impl Codec for EscapeCodec {
    fn run_codec(
        &self,
        mut input: &mut dyn std::io::Read,
        global_mode: CodecMode,
        _options: &Options,
        mut output: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        match global_mode {
            CodecMode::Encoding => {
                let mut writer = BytesToBytesEncoder::new(
                    &mut output,
                    Box::new(|buf| {
                        let result = std::str::from_utf8(buf)
                            .map_err(|err| {
                                std::io::Error::new(std::io::ErrorKind::InvalidData, err)
                            })?
                            .escape_default()
                            .to_string();
                        Ok((result.into_bytes(), &[]))
                    }),
                );

                std::io::copy(input, &mut writer)?;
                Ok(())
            }
            CodecMode::Decoding => {
                let mut reader = BytesToBytesDecoder::new(
                    &mut input,
                    Box::new(|buf| {
                        let str_ref =
                            std::str::from_utf8(buf).map_err(into_io_invalid_data_error)?;
                        // TODO: fix \ at the end
                        let string = format!("\"{}", str_ref);

                        let (result, remain) = match snailquote::unescape(&string) {
                            Ok(s) => (s.into_bytes(), Default::default()),
                            Err(err) => {
                                let index = match err {
                                    UnescapeError::InvalidEscape { index, .. } => index,
                                    UnescapeError::InvalidUnicode { index, .. } => index,
                                };
                                let (_, remain, idx) = split_at_char_index(str_ref, index - 1);
                                let result = snailquote::unescape(&string[..idx + 1]).unwrap();
                                (result.into_bytes(), remain.as_bytes())
                            } // Err(err) => {
                              //     return Err(std::io::Error::new(
                              //         std::io::ErrorKind::InvalidData,
                              //         err,
                              //     ))
                              // }
                        };

                        Ok((result, remain))
                    }),
                );

                std::io::copy(&mut reader, output)?;

                Ok(())
            }
        }
    }
}

fn split_at_char_index(string: &str, index: usize) -> (&str, &str, usize) {
    for (idx, (byte_idx, _)) in string.char_indices().enumerate() {
        if idx == index {
            let (pre, post) = string.split_at(byte_idx);
            return (pre, post, byte_idx);
        }
    }
    (string, "", string.len())
}

fn into_io_invalid_data_error<E>(err: E) -> std::io::Error
where
    E: Into<Box<dyn std::error::Error + Send + Sync>>,
{
    std::io::Error::new(std::io::ErrorKind::InvalidData, err)
}
