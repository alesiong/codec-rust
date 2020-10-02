use block_cipher::generic_array;
use block_modes::{block_cipher, block_padding};

use crate::{
    codecs::{Codec, CodecUsage, Options},
    utils::BytesToBytesEncoder,
};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub struct AesCodec {
    mode: AesMode,
}

enum AesMode {
    Cbc,
    Ecb,
}

const BLOCK_SIZE: usize = 16;

impl AesCodec {
    pub fn new_cbc() -> Self {
        AesCodec { mode: AesMode::Cbc }
    }

    pub fn new_ecb() -> Self {
        AesCodec { mode: AesMode::Ecb }
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
        global_mode: crate::codecs::CodecMode,
        options: &Options,
        output: &mut dyn std::io::Write,
    ) -> Result<()> {
        let key = options
            .get_text_raw("K")
            .ok_or(Box::<dyn std::error::Error>::from(
                "aes: missing required option key (-K)",
            ))?;
        let iv = match self.mode {
            AesMode::Cbc => {
                options
                    .get_text_raw("IV")
                    .ok_or(Box::<dyn std::error::Error>::from(
                        "aes[cbc]: missing required option iv (-IV)",
                    ))?
            }
            AesMode::Ecb => Default::default(),
        };

        match key.len() * 8 {
            128 => match self.mode {
                AesMode::Cbc => match global_mode {
                    crate::codecs::CodecMode::Encoding => {
                        encrypt::<block_modes::Cbc<_, _>, aes::Aes128>(&key, &iv, input, output)
                    }
                    crate::codecs::CodecMode::Decoding => todo!(),
                },
                AesMode::Ecb => match global_mode {
                    crate::codecs::CodecMode::Encoding => {
                        encrypt::<block_modes::Ecb<_, _>, aes::Aes128>(&key, &iv, input, output)
                    }
                    crate::codecs::CodecMode::Decoding => todo!(),
                },
            },
            192 => match self.mode {
                AesMode::Cbc => match global_mode {
                    crate::codecs::CodecMode::Encoding => {
                        encrypt::<block_modes::Cbc<_, _>, aes::Aes192>(&key, &iv, input, output)
                    }
                    crate::codecs::CodecMode::Decoding => todo!(),
                },
                AesMode::Ecb => match global_mode {
                    crate::codecs::CodecMode::Encoding => {
                        encrypt::<block_modes::Ecb<_, _>, aes::Aes192>(&key, &iv, input, output)
                    }
                    crate::codecs::CodecMode::Decoding => todo!(),
                },
            },
            256 => match self.mode {
                AesMode::Cbc => match global_mode {
                    crate::codecs::CodecMode::Encoding => {
                        encrypt::<block_modes::Cbc<_, _>, aes::Aes256>(&key, &iv, input, output)
                    }
                    crate::codecs::CodecMode::Decoding => todo!(),
                },
                AesMode::Ecb => match global_mode {
                    crate::codecs::CodecMode::Encoding => {
                        encrypt::<block_modes::Ecb<_, _>, aes::Aes256>(&key, &iv, input, output)
                    }
                    crate::codecs::CodecMode::Decoding => todo!(),
                },
            },
            _ => return Err(format!("invalid key length: {}bit", key.len() * 8).into()),
        }
    }
}

fn encrypt<M, C>(
    key: &[u8],
    iv: &[u8],
    input: &mut dyn std::io::Read,
    mut output: &mut dyn std::io::Write,
) -> Result<()>
where
    M: 'static + block_modes::BlockMode<C, block_padding::Pkcs7>,
    C: block_cipher::BlockCipher + block_cipher::NewBlockCipher,
{
    let mut cipher = M::new_var(key, iv)?;
    let mut writer = BytesToBytesEncoder::new(
        &mut output,
        Box::new(|buf| {
            let (blocks, remain) = buf.split_at(buf.len() - buf.len() % BLOCK_SIZE);
            Ok((encrypt_blocks(&mut cipher, blocks.to_vec()), remain))
        }),
    );
    std::io::copy(input, &mut writer)?;

    writer
        .finalize()
        .go(Box::new(|buf| Ok(Some(cipher.encrypt_vec(buf)))))?;
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