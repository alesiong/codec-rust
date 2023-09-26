use crate::codecs::MetaCodec;

#[derive(Default)]
pub struct UsageMetaCodec;

impl MetaCodec for UsageMetaCodec {
    fn run_meta_codec(
        &self,
        _input: &mut dyn std::io::Read,
        _global_mode: crate::codecs::CodecMode,
        options: &crate::codecs::Options,
        codec_meta_info: &crate::codecs::CodecMetaInfo,
        output: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        let codec_name = options.get_text_str("C")?;

        if let Some(codec_name) = codec_name {
            let codec = codec_meta_info
                .codecs_map
                .get(codec_name)
                .ok_or_else(|| anyhow::anyhow!("codec not found: {}", codec_name))?;
            writeln!(output, "{}", codec_name)?;

            if let Some(usage) = codec.as_codec_usage() {
                writeln!(output, "{}", usage.usage())?;
            }
        } else {
            writeln!(output, "Available codecs:")?;

            for (name, codec) in codec_meta_info.codecs_iter() {
                writeln!(output, "{}", name)?;

                if let Some(usage) = codec.as_codec_usage() {
                    writeln!(output, "{}", usage.usage())?;
                }
            }
        }

        Ok(())
    }
}
