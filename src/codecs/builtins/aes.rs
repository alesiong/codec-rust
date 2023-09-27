use std::io::{Read, Write};

use aes::cipher::{self, block_padding, generic_array, KeyInit, KeyIvInit};

use crate::{
    codecs::{Codec, CodecMode, CodecUsage, Options},
    utils::{BytesToBytesDecoder, BytesToBytesEncoder, DeathRattle},
};

pub struct AesCodec {
    mode: BlockCipherMode,
    cipher_type: BlockCipherType,
}

pub struct Sm4Codec(AesCodec);

enum BlockCipherMode {
    Cbc,
    Ecb,
}

enum BlockCipherType {
    Aes,
    Sm4,
}

impl AesCodec {
    pub fn new_cbc() -> Box<Self> {
        Box::new(AesCodec {
            mode: BlockCipherMode::Cbc,
            cipher_type: BlockCipherType::Aes,
        })
    }

    pub fn new_ecb() -> Box<Self> {
        Box::new(AesCodec {
            mode: BlockCipherMode::Ecb,
            cipher_type: BlockCipherType::Aes,
        })
    }

    fn do_block_mode<C>(
        &self,
        global_mode: CodecMode,
        key: &[u8],
        iv: &[u8],
        input: &mut dyn Read,
        output: &mut dyn Write,
    ) -> anyhow::Result<()>
    where
        C: 'static
            + cipher::BlockCipher
            + cipher::BlockEncryptMut
            + cipher::BlockDecryptMut
            + KeyInit,
    {
        match self.mode {
            BlockCipherMode::Cbc => match global_mode {
                CodecMode::Encoding => encrypt(
                    cbc::Encryptor::<C>::new_from_slices(key, iv)?,
                    input,
                    output,
                ),
                CodecMode::Decoding => decrypt(
                    cbc::Decryptor::<C>::new_from_slices(key, iv)?,
                    input,
                    output,
                ),
            },
            BlockCipherMode::Ecb => match global_mode {
                CodecMode::Encoding => {
                    encrypt(ecb::Encryptor::<C>::new_from_slice(key)?, input, output)
                }
                CodecMode::Decoding => {
                    decrypt(ecb::Decryptor::<C>::new_from_slice(key)?, input, output)
                }
            },
        }
    }
}

impl CodecUsage for AesCodec {
    fn usage(&self) -> String {
        match self.mode {
            BlockCipherMode::Cbc => "    -K key
    -IV iv
"
            .to_string(),
            BlockCipherMode::Ecb => "    -K key
"
            .to_string(),
        }
    }
}

impl Codec for AesCodec {
    fn run_codec(
        &self,
        input: &mut dyn std::io::Read,
        global_mode: CodecMode,
        options: &Options,
        output: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        let key = options.get_text_raw("K").ok_or_else(|| {
            anyhow::anyhow!(
                "{}: missing required option key (-K)",
                self.cipher_type.to_str()
            )
        })?;
        let iv = match self.mode {
            BlockCipherMode::Cbc => options.get_text_raw("IV").ok_or_else(|| {
                anyhow::anyhow!(
                    "{}[cbc]: missing required option iv (-IV)",
                    self.cipher_type.to_str()
                )
            })?,
            BlockCipherMode::Ecb => Default::default(),
        };

        match self.cipher_type {
            BlockCipherType::Aes => match key.len() * 8 {
                128 => self.do_block_mode::<aes::Aes128>(global_mode, key, iv, input, output),
                192 => self.do_block_mode::<aes::Aes192>(global_mode, key, iv, input, output),
                256 => self.do_block_mode::<aes::Aes256>(global_mode, key, iv, input, output),
                _ => anyhow::bail!("invalid key length: {}bit", key.len() * 8),
            },
            BlockCipherType::Sm4 => match key.len() * 8 {
                128 => self.do_block_mode::<sm4::Sm4>(global_mode, key, iv, input, output),
                _ => anyhow::bail!("invalid key length: {}bit", key.len() * 8),
            },
        }
    }

    fn as_codec_usage(&self) -> Option<&dyn CodecUsage> {
        Some(self)
    }
}

impl Sm4Codec {
    pub fn new_cbc() -> Box<Self> {
        Box::new(Sm4Codec(AesCodec {
            mode: BlockCipherMode::Cbc,
            cipher_type: BlockCipherType::Sm4,
        }))
    }

    pub fn new_ecb() -> Box<Self> {
        Box::new(Sm4Codec(AesCodec {
            mode: BlockCipherMode::Ecb,
            cipher_type: BlockCipherType::Sm4,
        }))
    }
}

impl CodecUsage for Sm4Codec {
    fn usage(&self) -> String {
        self.0.usage()
    }
}

impl Codec for Sm4Codec {
    fn run_codec(
        &self,
        input: &mut dyn std::io::Read,
        global_mode: CodecMode,
        options: &Options,
        output: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        self.0.run_codec(input, global_mode, options, output)
    }

    fn as_codec_usage(&self) -> Option<&dyn CodecUsage> {
        Some(self)
    }
}

impl BlockCipherType {
    fn to_str(&self) -> &'static str {
        match self {
            BlockCipherType::Aes => "aes",
            BlockCipherType::Sm4 => "sm4",
        }
    }
}

fn encrypt<M>(
    mut cipher: M,
    input: &mut dyn std::io::Read,
    mut output: &mut dyn std::io::Write,
) -> anyhow::Result<()>
where
    M: 'static + cipher::BlockEncryptMut,
{
    let block_size = M::block_size();

    let mut writer = BytesToBytesEncoder::new(&mut output, |buf| {
        let (blocks, remain) = buf.split_at(buf.len() - buf.len() % block_size);
        Ok((encrypt_blocks(&mut cipher, blocks.to_vec()), remain))
    });
    std::io::copy(input, &mut writer)?;

    writer.finalize().death_rattle(|buf| {
        Ok(Some(
            cipher.encrypt_padded_vec_mut::<block_padding::Pkcs7>(buf),
        ))
    })?;

    Ok(())
}

fn decrypt<M>(
    mut cipher: M,
    mut input: &mut dyn std::io::Read,
    mut output: &mut dyn std::io::Write,
) -> anyhow::Result<()>
where
    M: 'static + cipher::BlockDecryptMut, //  block_modes::BlockMode<C, block_padding::Pkcs7>,
{
    let block_size = M::block_size();

    let mut reader = BytesToBytesDecoder::new(
        &mut input,
        Box::new(|buf| {
            let (blocks, remain) = if buf.len() % block_size == 0 {
                buf.split_at(buf.len() - block_size)
            } else {
                buf.split_at(buf.len() - buf.len() % block_size)
            };
            Ok((decrypt_blocks(&mut cipher, blocks.to_vec()), remain))
        }),
    );

    reader.set_need_finalize(true);

    std::io::copy(&mut reader, output)?;

    reader.finalize().death_rattle((
        |buf| match cipher.decrypt_padded_vec_mut::<block_padding::Pkcs7>(buf) {
            Ok(r) => Ok(Some(r)),
            Err(err) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, err)),
        },
        &mut output,
    ))?;

    Ok(())
}

fn encrypt_blocks<M>(cipher: &mut M, mut plaintext_blocks: Vec<u8>) -> Vec<u8>
where
    M: 'static + cipher::BlockEncryptMut,
{
    cipher.encrypt_blocks_mut(to_blocks(&mut plaintext_blocks));
    plaintext_blocks
}

fn decrypt_blocks<M>(cipher: &mut M, mut ciphertext_blocks: Vec<u8>) -> Vec<u8>
where
    M: 'static + cipher::BlockDecryptMut,
{
    cipher.decrypt_blocks_mut(to_blocks(&mut ciphertext_blocks));
    ciphertext_blocks
}

fn to_blocks<N>(data: &mut [u8]) -> &mut [generic_array::GenericArray<u8, N>]
where
    N: generic_array::ArrayLength<u8>,
{
    let n = N::to_usize();
    debug_assert!(data.len() % n == 0);

    #[allow(unsafe_code)]
    unsafe {
        std::slice::from_raw_parts_mut(
            data.as_ptr() as *mut generic_array::GenericArray<u8, N>,
            data.len() / n,
        )
    }
}
