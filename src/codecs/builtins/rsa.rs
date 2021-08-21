use rand::rngs::OsRng;
use rsa::{PaddingScheme, PublicKey, RsaPrivateKey, RsaPublicKey};

use crate::codecs::Codec;

#[derive(Default)]
pub struct RsaCryptCodec;

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
            options.get_text_str("-PS")?.unwrap_or("oaep"),
            options.get_text_str("-H")?.unwrap_or("sha256"),
        )?;

        let mut buffer = Vec::<u8>::with_capacity(1024 * 8);

        let key_type = if options.get_switch("8") {
            KeyType::Pkcs8
        } else {
            KeyType::Pkcs1
        };

        match global_mode {
            crate::codecs::CodecMode::Encoding => {
                let pub_key_pem: String = options.get_text("PK")?.ok_or_else(|| {
                    anyhow::anyhow!("rsa: missing required option public key (-PK)")
                })?;

                let pub_key: RsaPublicKey = get_pub_key(&pub_key_pem, key_type)?;

                let mut rng = OsRng;

                std::io::copy(input, &mut buffer)?;

                output.write_all(&pub_key.encrypt(&mut rng, padding, &buffer)?)?;
            }
            crate::codecs::CodecMode::Decoding => {
                let pri_key_pem: String = options.get_text("SK")?.ok_or_else(|| {
                    anyhow::anyhow!("rsa: missing required option private key (-SK)")
                })?;
                let pri_key: RsaPrivateKey = get_pri_key(&pri_key_pem, key_type)?;

                std::io::copy(input, &mut buffer)?;

                output.write_all(&pri_key.decrypt(padding, &buffer)?)?;
            }
        }

        Ok(())
    }
}

enum KeyType {
    Pkcs1,
    Pkcs8,
}

fn get_pub_key(text: &str, key_type: KeyType) -> anyhow::Result<RsaPublicKey> {
    let key = match key_type {
        KeyType::Pkcs1 => rsa::pkcs1::FromRsaPublicKey::from_pkcs1_pem(text)?,
        KeyType::Pkcs8 => rsa::pkcs8::FromPublicKey::from_public_key_pem(text)?,
    };
    Ok(key)
}

fn get_pri_key(text: &str, key_type: KeyType) -> anyhow::Result<RsaPrivateKey> {
    let key = match key_type {
        KeyType::Pkcs1 => rsa::pkcs1::FromRsaPrivateKey::from_pkcs1_pem(text)?,
        KeyType::Pkcs8 => rsa::pkcs8::FromPrivateKey::from_pkcs8_pem(text)?,
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
