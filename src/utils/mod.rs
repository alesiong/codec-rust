pub struct MultiWriter<'a> {
    writers: Vec<&'a mut dyn std::io::Write>,
}

impl<'a> MultiWriter<'a> {
    pub fn new(writers: Vec<&'a mut dyn std::io::Write>) -> MultiWriter<'a> {
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
