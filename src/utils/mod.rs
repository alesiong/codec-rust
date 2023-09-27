use std::io::{Error, ErrorKind};

pub struct MultiWriter<'a> {
    writers: Vec<&'a mut dyn std::io::Write>,
}

impl<'a> MultiWriter<'a> {
    pub fn new(writers: Vec<&'a mut dyn std::io::Write>) -> MultiWriter<'a> {
        MultiWriter { writers }
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

// type Mapping<'a> = dyn 'a + FnMut(&[u8]) -> std::io::Result<(Vec<u8>, &[u8])>;
// trait Finalizer = FnOnce(&[u8]) -> std::io::Result<Option<Vec<u8>>>;

pub struct BytesToBytesEncoder<'w, W, M>
where
    W: std::io::Write,
{
    writer: Option<&'w mut W>,
    mapping: M,
    write_buffer: Vec<u8>,
}

impl<'w, W, M> BytesToBytesEncoder<'w, W, M>
where
    W: std::io::Write,
    M: FnMut(&[u8]) -> std::io::Result<(Vec<u8>, &[u8])>,
{
    pub fn new<'m>(writer: &'w mut W, mapping: M) -> Self
    where
        M: 'm,
    {
        BytesToBytesEncoder {
            writer: Some(writer),
            mapping,
            write_buffer: vec![],
        }
    }

    #[must_use]
    pub fn finalize(mut self) -> WriterDeathRattle<'w, W> {
        WriterDeathRattle {
            writer: self.writer.take().unwrap(),
            write_buffer: std::mem::take(&mut self.write_buffer),
        }
    }
}

impl<W, M> Drop for BytesToBytesEncoder<'_, W, M>
where
    W: std::io::Write,
{
    fn drop(&mut self) {
        if !self.write_buffer.is_empty() {
            panic!("bytes left without calling finalize");
        }
    }
}

pub trait DeathRattle<'a, Args, Ret> {
    fn death_rattle(self, arg: Args) -> Ret;
}

pub struct WriterDeathRattle<'w, W>
where
    W: std::io::Write,
{
    writer: &'w mut W,
    write_buffer: Vec<u8>,
}

impl<'w, W, F> DeathRattle<'w, F, std::io::Result<()>> for WriterDeathRattle<'w, W>
where
    W: std::io::Write,
    F: FnOnce(&[u8]) -> std::io::Result<Option<Vec<u8>>>,
{
    fn death_rattle(self, finalizer: F) -> std::io::Result<()> {
        if self.write_buffer.is_empty() {
            return Ok(());
        }
        if let Some(bytes) = finalizer(&self.write_buffer)? {
            return self.writer.write_all(&bytes);
        }
        Ok(())
    }
}

impl<W, M> std::io::Write for BytesToBytesEncoder<'_, W, M>
where
    W: std::io::Write,
    M: FnMut(&[u8]) -> std::io::Result<(Vec<u8>, &[u8])>,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut pre_write_buffer = std::mem::take(&mut self.write_buffer);

        pre_write_buffer.extend_from_slice(buf);

        let (to_write, remain) = (self.mapping)(&pre_write_buffer)?;

        self.write_buffer.extend_from_slice(remain);
        self.writer.as_mut().unwrap().write_all(&to_write)?;

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.as_mut().unwrap().flush()
    }
}

pub struct BytesToBytesDecoder<'a, R, M>
where
    R: 'a + std::io::Read,
{
    reader: &'a mut R,
    mapping: M,
    write_buffer: Vec<u8>,
    remain_buffer: Vec<u8>,
    index_remain: usize,
    buffer: [u8; 1024],
    need_finalize: bool,
}

impl<'a, R, M> BytesToBytesDecoder<'a, R, M>
where
    R: 'a + std::io::Read,
    M: FnMut(&[u8]) -> std::io::Result<(Vec<u8>, &[u8])>,
{
    pub fn new(reader: &'a mut R, mapping: M) -> Self {
        Self {
            reader,
            mapping,
            write_buffer: vec![],
            remain_buffer: vec![],
            index_remain: 0,
            buffer: [0; 1024],
            need_finalize: false,
        }
    }

    pub fn set_need_finalize(&mut self, need_finalize: bool) {
        self.need_finalize = need_finalize
    }

    #[must_use]
    pub fn finalize(mut self) -> ReaderDeathRattle {
        ReaderDeathRattle {
            write_buffer: std::mem::take(&mut self.write_buffer),
        }
    }

    fn remain_buffer_len(&self) -> usize {
        self.remain_buffer.len() - self.index_remain
    }
}

impl<'a, R, M> Drop for BytesToBytesDecoder<'a, R, M>
where
    R: 'a + std::io::Read,
{
    fn drop(&mut self) {
        if self.need_finalize && !self.write_buffer.is_empty() {
            panic!("bytes left without calling finalize");
        }
    }
}

pub struct ReaderDeathRattle {
    write_buffer: Vec<u8>,
}

impl<W, F> DeathRattle<'_, (F, &mut W), std::io::Result<()>> for ReaderDeathRattle
where
    W: std::io::Write,
    F: FnOnce(&[u8]) -> std::io::Result<Option<Vec<u8>>>,
{
    fn death_rattle(self, (finalizer, writer): (F, &mut W)) -> std::io::Result<()> {
        if self.write_buffer.is_empty() {
            return Ok(());
        }
        if let Some(bytes) = (finalizer)(&self.write_buffer)? {
            writer.write_all(&bytes)?;
        }
        Ok(())
    }
}

impl<'a, R, M> std::io::Read for BytesToBytesDecoder<'a, R, M>
where
    R: std::io::Read,
    M: FnMut(&[u8]) -> std::io::Result<(Vec<u8>, &[u8])>,
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
            if !self.need_finalize && !self.write_buffer.is_empty() {
                return Err(Error::new(
                    ErrorKind::UnexpectedEof,
                    "eof reached with bytes left",
                ));
            }
            return Ok(0);
        }

        let mut pre_write_buffer = std::mem::take(&mut self.write_buffer);
        pre_write_buffer.extend_from_slice(&self.buffer[..n]);

        let (result, remain) = (self.mapping)(&pre_write_buffer)?;
        self.write_buffer.extend_from_slice(remain);

        if result.is_empty() {
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
