use percent_encoding::AsciiSet;

use crate::{
    codecs::{Codec, CodecMode, CodecUsage, Options},
    utils::{BytesToBytesDecoder, BytesToBytesEncoder},
};

#[derive(Default)]
pub struct UrlCodec;

impl CodecUsage for UrlCodec {
    fn usage(&self) -> String {
        "    url query escape/unescape
    -p: use path escape instead of query escape"
            .to_string()
    }
}

impl Codec for UrlCodec {
    fn run_codec(
        &self,
        mut input: &mut dyn std::io::Read,
        global_mode: CodecMode,
        options: &Options,
        mut output: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        let escape = if options.get_switch("p") {
            escape_path
        } else {
            escape_query
        };

        match global_mode {
            CodecMode::Encoding => {
                let mut writer = BytesToBytesEncoder::new(&mut output, |buf| {
                    let result = escape(buf);
                    Ok((result.into_bytes(), &[]))
                });

                std::io::copy(input, &mut writer)?;
                Ok(())
            }
            CodecMode::Decoding => {
                let mut reader = BytesToBytesDecoder::new(
                    &mut input,
                    Box::new(|buf| {
                        let split_at = buf
                            .iter()
                            .rev()
                            .position(|&b| b == b'%')
                            .map(|rev_pos| buf.len() - rev_pos - 1)
                            .and_then(|pos| if buf.len() - pos < 3 { Some(pos) } else { None })
                            .unwrap_or(buf.len());

                        let (bytes, remain) = buf.split_at(split_at);

                        let result = percent_encoding::percent_decode(bytes).collect();
                        Ok((result, remain))
                    }),
                );

                std::io::copy(&mut reader, output)?;

                Ok(())
            }
        }
    }

    fn as_codec_usage(&self) -> Option<&dyn CodecUsage> {
        Some(self as &dyn CodecUsage)
    }
}

const QUERY_SET: &AsciiSet = &percent_encoding::NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~')
    .remove(b' ');

const PATH_SET: &AsciiSet = &percent_encoding::CONTROLS
    .add(b'%')
    .add(b'!')
    .add(b'\'')
    .add(b'(')
    .add(b')')
    .add(b'*')
    .add(b',')
    .add(b'/')
    .add(b';')
    .add(b'[')
    .add(b'\\')
    .add(b']')
    .add(b'^')
    .add(b'|')
    .add(b' ')
    .add(b'"')
    .add(b'#')
    .add(b'<')
    .add(b'>')
    .add(b'?')
    .add(b'`')
    .add(b'{')
    .add(b'}');

fn escape_query(bytes: &[u8]) -> String {
    percent_encoding::percent_encode(bytes, QUERY_SET)
        .to_string()
        .replace(' ', "+")
}

fn escape_path(bytes: &[u8]) -> String {
    percent_encoding::percent_encode(bytes, PATH_SET).to_string()
}
