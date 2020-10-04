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

type Mapping<'a> = dyn 'a + FnMut(&[u8]) -> std::io::Result<(Vec<u8>, &[u8])>;
// trait Finalizer = FnOnce(&[u8]) -> std::io::Result<Option<Vec<u8>>>;

pub struct BytesToBytesEncoder<'w, 'm, W>
where
    W: 'w + std::io::Write,
{
    writer: Option<&'w mut W>,
    mapping: Box<Mapping<'m>>,
    write_buffer: Vec<u8>,
}

impl<'w, 'm, W> BytesToBytesEncoder<'w, 'm, W>
where
    W: std::io::Write,
{
    pub fn new(writer: &'w mut W, mapping: Box<Mapping<'m>>) -> Self {
        BytesToBytesEncoder {
            writer: Some(writer),
            mapping: mapping,
            write_buffer: vec![],
        }
    }

    pub fn finalize<F>(mut self) -> impl DeathRattle<'w, F, std::io::Result<()>>
    where
        // type F should be inferred from the next death_rattle() call
        F: FnOnce(&[u8]) -> std::io::Result<Option<Vec<u8>>>,
    {
        WriterDeathRattle {
            writer: self.writer.take().unwrap(),
            write_buffer: std::mem::replace(&mut self.write_buffer, vec![]),
        }
    }
}

impl<'w, 'm, W> Drop for BytesToBytesEncoder<'w, 'm, W>
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

struct WriterDeathRattle<'w, W>
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

impl<W> std::io::Write for BytesToBytesEncoder<'_, '_, W>
where
    W: std::io::Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut pre_write_buffer = std::mem::replace(&mut self.write_buffer, vec![]);

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

pub struct BytesToBytesDecoder<'a, R>
where
    R: 'a + std::io::Read,
{
    reader: &'a mut R,
    mapping: Box<Mapping<'a>>,
    write_buffer: Vec<u8>,
    remain_buffer: Vec<u8>,
    index_remain: usize,
    buffer: [u8; 1024],
    need_finalize: bool,
}

impl<'a, R> BytesToBytesDecoder<'a, R>
where
    R: 'a + std::io::Read,
{
    pub fn new(reader: &'a mut R, mapping: Box<Mapping<'a>>) -> Self {
        Self {
            reader: reader,
            mapping: mapping,
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

    pub fn finalize<'w, W, F>(
        mut self,
    ) -> impl DeathRattle<'static, (F, &'w mut W), std::io::Result<()>>
    where
        W: std::io::Write,
        // type F should be inferred from the next death_rattle() call
        F: FnOnce(&[u8]) -> std::io::Result<Option<Vec<u8>>>,
    {
        ReaderDeathRattle {
            write_buffer: std::mem::replace(&mut self.write_buffer, vec![]),
        }
    }

    fn remain_buffer_len(&self) -> usize {
        self.remain_buffer.len() - self.index_remain
    }
}

impl<'a, R> Drop for BytesToBytesDecoder<'a, R>
where
    R: 'a + std::io::Read,
{
    fn drop(&mut self) {
        if self.need_finalize && !self.write_buffer.is_empty() {
            panic!("bytes left without calling finalize");
        }
    }
}

struct ReaderDeathRattle {
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

impl<R> std::io::Read for BytesToBytesDecoder<'_, R>
where
    R: std::io::Read,
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

        let mut pre_write_buffer = std::mem::replace(&mut self.write_buffer, vec![]);
        pre_write_buffer.extend_from_slice(&self.buffer[..n]);

        let (result, remain) = (&mut self.mapping)(&pre_write_buffer)?;
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
