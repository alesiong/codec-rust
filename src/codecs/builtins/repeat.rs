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

        let mut buffer = bytebuffer::ByteBuffer::new();

        if times > 0 {
            let mut writer = MultiWriter::new(vec![output, &mut buffer]);
            let _ = std::io::copy(input, &mut writer)?;

            for _ in 1..times {
                let _ = output.write_all(&buffer.to_bytes())?;
            }
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

struct MultiWriter<'a> {
    writers: Vec<&'a mut dyn std::io::Write>,
}

impl<'a> MultiWriter<'a> {
    fn new(writers: Vec<&'a mut dyn std::io::Write>) -> MultiWriter<'a> {
        MultiWriter { writers: writers }
    }
}

impl std::io::Write for MultiWriter<'_> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        for writer in &mut self.writers {
            let n = writer.write(buf)?;
            if n != buf.len() {
                return Err(std::io::Error::new(
                    std::io::ErrorKind::Interrupted,
                    "short write",
                ));
            }
        }
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        for writer in &mut self.writers {
            writer.flush()?
        }
        Ok(())
    }
}
