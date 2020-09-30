use crate::codecs::{Codec, CodecUsage};

#[derive(Default)]

pub struct RepeatCodecs;

#[derive(Default)]
pub struct IdCodecs(RepeatCodecs);

impl CodecUsage for RepeatCodecs {
    fn usage(&self) -> String {
        "    -T times: repeat input for `times` times (int, >=0, default 0)".to_string()
    }
}

impl Codec for RepeatCodecs {
    fn run_codec(
        &self,
        input: &mut dyn std::io::Read,
        _global_mode: crate::codecs::CodecMode,
        options: &std::collections::HashMap<String, String>,
        output: &mut dyn std::io::Write,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut times = 0;
        if let Some(t) = options.get("T") {
            times = t.parse()?;
            if times < 0 {
                return Err("repeat: times cannot be minus".into());
            }
        }

        let mut buffer: Vec<u8> = vec![];

        for i in 0..times {
            if i == 0 {
                let _ = input.read_to_end(&mut buffer)?;
            }
            let _ = output.write_all(&buffer)?;
        }

        Ok(())
    }
}

impl CodecUsage for IdCodecs {
    fn usage(&self) -> String {
        "    pass input to output as is".to_string()
    }
}

impl Codec for IdCodecs {
    fn run_codec(
        &self,
        input: &mut dyn std::io::Read,
        global_mode: crate::codecs::CodecMode,
        _options: &std::collections::HashMap<String, String>,
        output: &mut dyn std::io::Write,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let mut options = std::collections::HashMap::new();
        options.insert("T".to_string(), "1".to_string());

        self.0.run_codec(input, global_mode, &options, output)
    }
}
