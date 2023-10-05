use crate::{
    codecs::{Codec, CodecMode, CodecUsage, Options},
    utils::BytesToBytesEncoder,
};

use digest::Digest;

pub struct HashCodec {
    hash_type: HashType,
}

enum HashType {
    Md5,
    #[cfg(feature = "libc")]
    Sha256,
    Sm3,
}

impl HashCodec {
    pub fn new_md5() -> Box<Self> {
        Box::new(HashCodec {
            hash_type: HashType::Md5,
        })
    }

    #[cfg(feature = "libc")]
    pub fn new_sha256() -> Box<Self> {
        Box::new(HashCodec {
            hash_type: HashType::Sha256,
        })
    }

    pub fn new_sm3() -> Box<Self> {
        Box::new(HashCodec {
            hash_type: HashType::Sm3,
        })
    }
}

impl Codec for HashCodec {
    fn run_codec(
        &self,
        input: &mut dyn std::io::Read,
        global_mode: CodecMode,
        _options: &Options,
        output: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        match global_mode {
            CodecMode::Encoding => match self.hash_type {
                HashType::Md5 => digest::<md5::Md5>(input, output),
                #[cfg(feature = "libc")]
                HashType::Sha256 => digest::<sha2::Sha256>(input, output),
                HashType::Sm3 => digest::<sm3::Sm3>(input, output),
            },
            CodecMode::Decoding => Err(anyhow::anyhow!("hash: cannot decode")),
        }
    }
    fn as_codec_usage(&self) -> Option<&dyn CodecUsage> {
        Some(self)
    }
}

impl CodecUsage for HashCodec {
    fn usage(&self) -> String {
        "    calculate hash digest\n".to_string()
    }
}

fn digest<D: Digest>(
    input: &mut dyn std::io::Read,
    mut output: &mut dyn std::io::Write,
) -> anyhow::Result<()> {
    let mut hasher = D::new();

    let mut writer = BytesToBytesEncoder::new(&mut output, |buf| {
        hasher.update(buf);
        Ok(Default::default())
    });

    std::io::copy(input, &mut writer)?;

    drop(writer);

    output.write_all(&hasher.finalize())?;

    Ok(())
}
