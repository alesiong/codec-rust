use crate::codecs::{Codec, Options};

#[derive(Default)]
pub struct CatCodecs;

impl Codec for CatCodecs {
    fn run_codec(
        &self,
        input: &mut dyn std::io::Read,
        _global_mode: crate::codecs::CodecMode,
        options: &Options,
        output: &mut dyn std::io::Write,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let input_file =
            options
                .get_text::<String>("F")?
                .ok_or(Box::<dyn std::error::Error>::from(
                    "cat: missing required option input file (-F)",
                ))?;

        if !options.get_switch("c") {
            let _ = std::io::copy(input, output)?;
        }

        let mut file = std::fs::File::open(input_file)?;
        let _ = std::io::copy(&mut file, output)?;

        Ok(())
    }
}
