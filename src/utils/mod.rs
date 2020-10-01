use std::io::{Error, ErrorKind};

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
                return Err(Error::new(ErrorKind::Interrupted, "short write"));
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

pub struct BytesToBytesEncoder<'a, W, F>
where
    W: 'a + std::io::Write,
    F: Fn(&[u8]) -> Vec<u8>,
{
    writer: &'a mut W,
    mapping: F,
}

impl<'a, W, F> BytesToBytesEncoder<'a, W, F>
where
    W: std::io::Write,
    F: Fn(&[u8]) -> Vec<u8>,
{
    pub fn new(writer: &'a mut W, mapping: F) -> Self {
        BytesToBytesEncoder {
            writer: writer,
            mapping: mapping,
        }
    }
}

impl<W, F> std::io::Write for BytesToBytesEncoder<'_, W, F>
where
    W: std::io::Write,
    F: Fn(&[u8]) -> Vec<u8>,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let to_write = &(self.mapping)(buf);
        self.writer.write_all(to_write)?;

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}

pub struct BytesToBytesDecoder<'a, R, F>
where
    R: 'a + std::io::Read,
    F: Fn(&[u8]) -> std::io::Result<(Vec<u8>, &[u8])>,
{
    reader: &'a mut R,
    mapping: F,
    write_buffer: Vec<u8>,
    remain_buffer: Vec<u8>,
    index_remain: usize,
    buffer: [u8; 1024],
}

impl<'a, R, F> BytesToBytesDecoder<'a, R, F>
where
    R: 'a + std::io::Read,
    F: Fn(&[u8]) -> std::io::Result<(Vec<u8>, &[u8])>,
{
    pub fn new(reader: &'a mut R, mapping: F) -> Self {
        Self {
            reader: reader,
            mapping: mapping,
            write_buffer: vec![],
            remain_buffer: vec![],
            index_remain: 0,
            buffer: [0; 1024],
        }
    }

    fn remain_buffer_len(&self) -> usize {
        self.remain_buffer.len() - self.index_remain
    }
}

impl<R, F> std::io::Read for BytesToBytesDecoder<'_, R, F>
where
    R: std::io::Read,
    F: Fn(&[u8]) -> std::io::Result<(Vec<u8>, &[u8])>,
{
    fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> {
        if self.remain_buffer_len() > 0 {
            let n = (&self.remain_buffer[self.index_remain..]).read(buf)?;
            self.index_remain += n;
            return Ok(n);
        }

        self.index_remain = 0;
        self.remain_buffer.clear();

        let n = self.reader.read(&mut self.buffer)?;
        if n == 0 {
            if !self.write_buffer.is_empty() {
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "eof reached with bytes left",
                ));
            }
            return Ok(0);
        }

        let mut pre_write_buffer = std::mem::replace(&mut self.write_buffer, vec![]);
        pre_write_buffer.extend_from_slice(&self.buffer[..n]);

        let (result, remain) = (&self.mapping)(&pre_write_buffer)?;
        self.write_buffer.extend_from_slice(remain);

        if result.len() == 0 {
            return Err(Error::new(
                ErrorKind::Interrupted,
                "no enough bytes to write",
            ));
        }

        let n = result.as_slice().read(buf)?;
        self.remain_buffer.extend_from_slice(&result[n..]);

        Ok(n)
    }
}
