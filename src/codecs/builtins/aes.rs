use block_cipher::generic_array;
use block_modes::{block_cipher, block_padding};

use crate::{
    codecs::{Codec, CodecMode, CodecUsage, Options},
    utils::{BytesToBytesDecoder, BytesToBytesEncoder},
};

pub struct AesCodec {
    mode: AesMode,
}

enum AesMode {
    Cbc,
    Ecb,
}

impl AesCodec {
    pub fn new_cbc() -> Box<Self> {
        Box::new(AesCodec { mode: AesMode::Cbc })
    }

    pub fn new_ecb() -> Box<Self> {
        Box::new(AesCodec { mode: AesMode::Ecb })
    }
}

impl CodecUsage for AesCodec {
    fn usage(&self) -> String {
        match self.mode {
            AesMode::Cbc => "    -K key
    -IV iv
"
            .to_string(),
            AesMode::Ecb => "    -K key
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
        let key = options
            .get_text_raw("K")
            .ok_or(anyhow::anyhow!("aes: missing required option key (-K)"))?;
        let iv = match self.mode {
            AesMode::Cbc => options.get_text_raw("IV").ok_or(anyhow::anyhow!(
                "aes[cbc]: missing required option iv (-IV)",
            ))?,
            AesMode::Ecb => Default::default(),
        };

        match key.len() * 8 {
            128 => match self.mode {
                AesMode::Cbc => {
                    let cipher = new_cipher::<block_modes::Cbc<_, _>, aes::Aes128>(&key, &iv)?;
                    match global_mode {
                        CodecMode::Encoding => encrypt(cipher, input, output),
                        CodecMode::Decoding => decrypt(cipher, input, output),
                    }
                }
                AesMode::Ecb => {
                    let cipher = new_cipher::<block_modes::Ecb<_, _>, aes::Aes128>(&key, &iv)?;
                    match global_mode {
                        CodecMode::Encoding => encrypt(cipher, input, output),
                        CodecMode::Decoding => decrypt(cipher, input, output),
                    }
                }
            },
            192 => match self.mode {
                AesMode::Cbc => {
                    let cipher = new_cipher::<block_modes::Cbc<_, _>, aes::Aes192>(&key, &iv)?;
                    match global_mode {
                        CodecMode::Encoding => encrypt(cipher, input, output),
                        CodecMode::Decoding => decrypt(cipher, input, output),
                    }
                }
                AesMode::Ecb => {
                    let cipher = new_cipher::<block_modes::Ecb<_, _>, aes::Aes192>(&key, &iv)?;
                    match global_mode {
                        CodecMode::Encoding => encrypt(cipher, input, output),
                        CodecMode::Decoding => decrypt(cipher, input, output),
                    }
                }
            },
            256 => match self.mode {
                AesMode::Cbc => {
                    let cipher = new_cipher::<block_modes::Cbc<_, _>, aes::Aes256>(&key, &iv)?;
                    match global_mode {
                        CodecMode::Encoding => encrypt(cipher, input, output),
                        CodecMode::Decoding => decrypt(cipher, input, output),
                    }
                }
                AesMode::Ecb => {
                    let cipher = new_cipher::<block_modes::Ecb<_, _>, aes::Aes256>(&key, &iv)?;
                    match global_mode {
                        CodecMode::Encoding => encrypt(cipher, input, output),
                        CodecMode::Decoding => decrypt(cipher, input, output),
                    }
                }
            },
            _ => anyhow::bail!("invalid key length: {}bit", key.len() * 8),
        }
    }
}

fn new_cipher<M, C>(key: &[u8], iv: &[u8]) -> anyhow::Result<M>
where
    M: 'static + block_modes::BlockMode<C, block_padding::Pkcs7>,
    C: block_cipher::BlockCipher + block_cipher::NewBlockCipher,
{
    Ok(M::new_var(key, iv)?)
}

fn encrypt<M, C>(
    mut cipher: M,
    input: &mut dyn std::io::Read,
    mut output: &mut dyn std::io::Write,
) -> anyhow::Result<()>
where
    M: 'static + block_modes::BlockMode<C, block_padding::Pkcs7>,
    C: block_cipher::BlockCipher + block_cipher::NewBlockCipher,
{
    let block_size = block_size::<C>();

    let mut writer = BytesToBytesEncoder::new(&mut output, |buf| {
        let (blocks, remain) = buf.split_at(buf.len() - buf.len() % block_size);
        Ok((encrypt_blocks(&mut cipher, blocks.to_vec()), remain))
    });
    std::io::copy(input, &mut writer)?;

    writer
        .finalize()
        .death_rattle(|buf| Ok(Some(cipher.encrypt_vec(buf))))?;

    Ok(())
}

fn decrypt<M, C>(
    mut cipher: M,
    mut input: &mut dyn std::io::Read,
    mut output: &mut dyn std::io::Write,
) -> anyhow::Result<()>
where
    M: 'static + block_modes::BlockMode<C, block_padding::Pkcs7>,
    C: block_cipher::BlockCipher + block_cipher::NewBlockCipher,
{
    let block_size = block_size::<C>();

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
        |buf| match cipher.decrypt_vec(buf) {
            Ok(r) => Ok(Some(r)),
            Err(err) => Err(std::io::Error::new(std::io::ErrorKind::InvalidData, err)),
        },
        &mut output,
    ))?;

    Ok(())
}

fn encrypt_blocks<M, C>(cipher: &mut M, mut plaintext_blocks: Vec<u8>) -> Vec<u8>
where
    M: block_modes::BlockMode<C, block_padding::Pkcs7>,
    C: block_cipher::BlockCipher + block_cipher::NewBlockCipher,
{
    cipher.encrypt_blocks(to_blocks(&mut plaintext_blocks));
    plaintext_blocks
}

fn decrypt_blocks<M, C>(cipher: &mut M, mut ciphertext_blocks: Vec<u8>) -> Vec<u8>
where
    M: block_modes::BlockMode<C, block_padding::Pkcs7>,
    C: block_cipher::BlockCipher + block_cipher::NewBlockCipher,
{
    cipher.decrypt_blocks(to_blocks(&mut ciphertext_blocks));
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

fn block_size<C>() -> usize
where
    C: block_cipher::BlockCipher,
{
    <C::BlockSize as generic_array::typenum::Unsigned>::to_usize()
}
