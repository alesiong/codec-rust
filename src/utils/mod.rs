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
type Finalizer = dyn FnMut(&[u8]) -> std::io::Result<Option<Vec<u8>>>;
type FinalizerOnce = dyn FnOnce(&[u8]) -> std::io::Result<Option<Vec<u8>>>;

pub struct BytesToBytesEncoder<'w, 'm, W>
where
    W: 'w + std::io::Write,
{
    writer: &'w mut W,
    mapping: Box<Mapping<'m>>,
    write_buffer: Vec<u8>,
}

impl<'w, 'm, W> BytesToBytesEncoder<'w, 'm, W>
where
    W: std::io::Write,
{
    pub fn new(writer: &'w mut W, mapping: Box<Mapping<'m>>) -> Self {
        BytesToBytesEncoder {
            writer: writer,
            mapping: mapping,
            write_buffer: vec![],
        }
    }

    pub fn finalize(self) -> impl DeathRattle<'w, Box<FinalizerOnce>, std::io::Result<()>> {
        WriterDeathRattle {
            writer: self.writer,
            write_buffer: self.write_buffer,
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
impl<'w, W> DeathRattle<'w, Box<FinalizerOnce>, std::io::Result<()>> for WriterDeathRattle<'w, W>
where
    W: std::io::Write,
{
    fn death_rattle(self, finalizer: Box<FinalizerOnce>) -> std::io::Result<()> {
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
        self.writer.write_all(&to_write)?;

        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
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
    finalizer: Box<Finalizer>,
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
            finalizer: Box::new(|_| Ok(None)),
        }
    }

    pub fn set_finallizer(&mut self, finalizer: Box<Finalizer>) {
        self.finalizer = finalizer;
    }

    fn remain_buffer_len(&self) -> usize {
        self.remain_buffer.len() - self.index_remain
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
            if !self.write_buffer.is_empty() {
                if let Some(bytes) = (self.finalizer)(&self.write_buffer)? {
                    self.remain_buffer = bytes;
                    self.index_remain = self.remain_buffer.as_slice().read(buf)?;
                    return Ok(self.index_remain);
                } else {
                    return Err(Error::new(
                        ErrorKind::UnexpectedEof,
                        "eof reached with bytes left",
                    ));
                }
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
