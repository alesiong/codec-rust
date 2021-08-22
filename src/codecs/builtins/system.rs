use std::{io::ErrorKind, process::Stdio};

use tokio::{
    io::{AsyncRead, AsyncWrite},
    process::Command,
};

use crate::codecs::Codec;

#[derive(Default)]
pub struct SystemCodec;

impl Codec for SystemCodec {
    fn run_codec(
        &self,
        input: &mut dyn std::io::Read,
        _global_mode: crate::codecs::CodecMode,
        options: &crate::codecs::Options,
        output: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        let command_name = options
            .get_text_str("C")?
            .ok_or_else(|| anyhow::anyhow!("system: missing required option command name (-C)"))?;

        let args = options
            .get_text_str("A")?
            .unwrap_or("")
            .split(' ')
            .filter(|s| !s.is_empty());

        {
            let mut input = AsyncReadWrapper(input);
            let mut output = AsyncWriteWrapper(output);

            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_io()
                .build()?;

            rt.block_on(async {
                let mut child = Command::new(command_name)
                    .args(args)
                    .stdin(Stdio::piped())
                    .stdout(Stdio::piped())
                    .stderr(Stdio::inherit())
                    .spawn()?;

                let mut stdin = child
                    .stdin
                    .take()
                    .ok_or_else(|| anyhow::anyhow!("system: failed to open stdin"))?;

                let mut stdout = child
                    .stdout
                    .take()
                    .ok_or_else(|| anyhow::anyhow!("system: failed to open stdout"))?;

                let read_handle = async {
                    let _ = tokio::io::copy(&mut input, &mut stdin).await?;
                    drop(stdin);
                    anyhow::Result::<_>::Ok(())
                };

                let write_handle = async {
                    let _ = tokio::io::copy(&mut stdout, &mut output).await?;
                    drop(stdout);
                    anyhow::Result::<_>::Ok(())
                };

                let child_handle = async {
                    let _ = child.wait().await?;
                    anyhow::Result::<_>::Ok(())
                };

                tokio::try_join!(write_handle, read_handle, child_handle)?;
                anyhow::Result::<_>::Ok(())
            })?;
        }

        Ok(())
    }
}

struct AsyncReadWrapper<'a>(&'a mut dyn std::io::Read);

// impl AsyncReadWrapper<'_> {
//     unsafe fn new_static<'a>(r: &'a mut dyn std::io::Read) -> AsyncReadWrapper<'static> {
//         std::mem::transmute(AsyncReadWrapper(r))
//     }
// }

impl<'a> AsyncRead for AsyncReadWrapper<'_> {
    fn poll_read(
        self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        // TODO: maybe use uninitialized
        match self.get_mut().0.read(buf.initialize_unfilled()) {
            Ok(0) => std::task::Poll::Ready(Ok(())),
            Ok(len) => {
                buf.advance(len);

                std::task::Poll::Ready(Ok(()))
            }
            Err(ref e) if e.kind() == ErrorKind::Interrupted => std::task::Poll::Pending,
            Err(e) => std::task::Poll::Ready(Err(e)),
        }
    }
}

struct AsyncWriteWrapper<'a>(&'a mut dyn std::io::Write);

// impl AsyncWriteWrapper<'_> {
//     unsafe fn new_static<'a>(w: &'a mut dyn std::io::Write) -> AsyncWriteWrapper<'static> {
//         std::mem::transmute(AsyncWriteWrapper(w))
//     }
// }

impl<'a> AsyncWrite for AsyncWriteWrapper<'_> {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        std::task::Poll::Ready(self.0.write(buf))
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        std::task::Poll::Ready(self.0.flush())
    }

    fn poll_shutdown(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        self.poll_flush(cx)
    }

    fn poll_write_vectored(
        mut self: std::pin::Pin<&mut Self>,
        _cx: &mut std::task::Context<'_>,
        bufs: &[std::io::IoSlice<'_>],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        std::task::Poll::Ready(self.0.write_vectored(bufs))
    }

    fn is_write_vectored(&self) -> bool {
        true
    }
}
