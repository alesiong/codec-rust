use rand::rngs::OsRng;
use rsa::{PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey};

use crate::codecs::Codec;

#[derive(Default)]
pub struct RsaCryptCodec;

#[derive(Default)]
pub struct RsaSignCodec;

impl Codec for RsaCryptCodec {
    fn run_codec(
        &self,
        input: &mut dyn std::io::Read,
        global_mode: crate::codecs::CodecMode,
        options: &crate::codecs::Options,
        output: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        // TODO: read der
        let padding = get_padding_scheme(
            options.get_text_str("PS")?.unwrap_or("oaep"),
            options.get_text_str("H")?.unwrap_or("sha256"),
        )?;

        let mut buffer = Vec::<u8>::with_capacity(1024 * 8);

        match global_mode {
            crate::codecs::CodecMode::Encoding => {
                let pub_key: RsaPublicKey = get_pub_key(options)?;

                let mut rng = OsRng;

                std::io::copy(input, &mut buffer)?;

                output.write_all(&pub_key.encrypt(&mut rng, padding, &buffer)?)?;
            }
            crate::codecs::CodecMode::Decoding => {
                let pri_key: RsaPrivateKey = get_pri_key(options)?;

                std::io::copy(input, &mut buffer)?;

                output.write_all(&pri_key.decrypt(padding, &buffer)?)?;
            }
        }

        Ok(())
    }
}

impl Codec for RsaSignCodec {
    fn run_codec(
        &self,
        input: &mut dyn std::io::Read,
        global_mode: crate::codecs::CodecMode,
        options: &crate::codecs::Options,
        output: &mut dyn std::io::Write,
    ) -> anyhow::Result<()> {
        let mut buffer = Vec::<u8>::with_capacity(1024 * 8);
        let hash = options
            .get_text_str("H")?
            .map(|hash| match hash {
                "sha1" => Ok(rsa::Hash::SHA1),
                "sha256" => Ok(rsa::Hash::SHA2_256),
                _ => anyhow::bail!("invalid hash type: {}", hash),
            })
            .transpose()?;
        let padding = PaddingScheme::new_pkcs1v15_sign(hash);

        match global_mode {
            crate::codecs::CodecMode::Encoding => {
                let pri_key: RsaPrivateKey = get_pri_key(options)?;

                std::io::copy(input, &mut buffer)?;

                output.write_all(&pri_key.sign(padding, &buffer)?)?;
            }
            crate::codecs::CodecMode::Decoding => {
                let pub_key: RsaPublicKey = get_pub_key(options)?;

                let signature = options.get_text_raw("S").ok_or_else(|| {
                    anyhow::anyhow!("rsa: missing required option signature (-S)")
                })?;

                std::io::copy(input, &mut buffer)?;

                pub_key.verify(padding, &buffer, signature)?;
            }
        }

        Ok(())
    }
}

enum KeyType {
    Pkcs1,
    Pkcs8,
}

fn get_key_type(options: &crate::codecs::Options) -> KeyType {
    if options.get_switch("8") {
        KeyType::Pkcs8
    } else {
        KeyType::Pkcs1
    }
}

fn get_pub_key(options: &crate::codecs::Options) -> anyhow::Result<RsaPublicKey> {
    let text: String = options
        .get_text("PK")?
        .ok_or_else(|| anyhow::anyhow!("rsa: missing required option public key (-PK)"))?;
    let key = match get_key_type(options) {
        KeyType::Pkcs1 => rsa::pkcs1::FromRsaPublicKey::from_pkcs1_pem(&text)?,
        KeyType::Pkcs8 => rsa::pkcs8::FromPublicKey::from_public_key_pem(&text)?,
    };
    Ok(key)
}

fn get_pri_key(options: &crate::codecs::Options) -> anyhow::Result<RsaPrivateKey> {
    let text: String = options
        .get_text("SK")?
        .ok_or_else(|| anyhow::anyhow!("rsa: missing required option private key (-SK)"))?;
    let key = match get_key_type(options) {
        KeyType::Pkcs1 => rsa::pkcs1::FromRsaPrivateKey::from_pkcs1_pem(&text)?,
        KeyType::Pkcs8 => rsa::pkcs8::FromPrivateKey::from_pkcs8_pem(&text)?,
    };
    Ok(key)
}

fn get_padding_scheme(padding: &str, hash: &str) -> anyhow::Result<PaddingScheme> {
    let scheme = match padding {
        "oaep" => match hash {
            "sha1" => PaddingScheme::new_oaep::<sha1::Sha1>(),
            "sha256" => PaddingScheme::new_oaep::<sha2::Sha256>(),
            _ => anyhow::bail!("invalid hash type: {}", hash),
        },
        "pkcs15" => PaddingScheme::new_pkcs1v15_encrypt(),

        _ => anyhow::bail!("invalid padding scheme: {}", padding),
    };
    Ok(scheme)
}
