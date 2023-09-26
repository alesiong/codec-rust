use rand::rngs::OsRng;
use rsa::{
    Oaep,
    pkcs1::{DecodeRsaPrivateKey, DecodeRsaPublicKey},
    Pkcs1v15Encrypt, Pkcs1v15Sign, pkcs8::{DecodePrivateKey, DecodePublicKey}, RsaPrivateKey, RsaPublicKey,
};
use rsa::rand_core::CryptoRngCore;
use rsa::traits::PaddingScheme;

use crate::codecs::{Codec, CodecUsage};

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
    fn as_codec_usage(&self) -> Option<&dyn CodecUsage> {
        Some(self)
    }
}

impl CodecUsage for RsaCryptCodec {
    fn usage(&self) -> String {
        "    rsa encryption with public key and decryption with private key
    -PK pub_key: public key pem string, default pkcs1 format
    -SK pri_key: private key pem string, default pkcs1 format
    -8: use pkcs8 key format instead of pkcs1
    -dr: use der format instead of pem
    -PS scheme: padding scheme (oaep, pkcs15; defaults to oaep)
    -H algorithm: hash algorithm used for oaep padding scheme (sha1, sha256; defaults to sha256)
"
        .to_string()
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
        let padding = options
            .get_text_str("H")?
            .map(|hash| match hash {
                "sha1" => Ok(Pkcs1v15Sign::new::<sha1::Sha1>()),
                "sha256" => Ok(Pkcs1v15Sign::new::<sha2::Sha256>()),
                _ => anyhow::bail!("invalid hash type: {}", hash),
            })
            .transpose()?
            .unwrap_or_else(Pkcs1v15Sign::new_unprefixed);

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
    fn as_codec_usage(&self) -> Option<&dyn CodecUsage> {
        Some(self)
    }
}

impl CodecUsage for RsaSignCodec {
    fn usage(&self) -> String {
        "    rsa sign with private key and verification with public key
    NOTE:
        1. input must first be hashed in algorithm specified in -H option
            e.g. sha256 rsa-sign -SK sk_string -H sha256
        2. for verification, output nothing if succeeded, error if not (pending to change along with new `if` meta codec)
    -PK pub_key: public key pem string or der bytes, default pkcs1 format
    -SK pri_key: private key pem string or der bytes, default pkcs1 format
    -8: use pkcs8 key format instead of pkcs1
    -dr: use der format instead of pem
    -H algorithm: hash algorithm used for sign (sha1, sha256)
"
            .to_string()
    }
}

enum KeyType {
    Pkcs1,
    Pkcs8,
}

enum KeyEncoding {
    Pem,
    Der,
}

fn get_key_type_and_encoding(options: &crate::codecs::Options) -> (KeyType, KeyEncoding) {
    let key_type = if options.get_switch("8") {
        KeyType::Pkcs8
    } else {
        KeyType::Pkcs1
    };
    let key_encoding = if options.get_switch("dr") {
        KeyEncoding::Der
    } else {
        KeyEncoding::Pem
    };
    (key_type, key_encoding)
}

fn get_pub_key(options: &crate::codecs::Options) -> anyhow::Result<RsaPublicKey> {
    let text = options
        .get_text_raw("PK")
        .ok_or_else(|| anyhow::anyhow!("rsa: missing required option public key (-PK)"))?;

    let key = match get_key_type_and_encoding(options) {
        (KeyType::Pkcs1, KeyEncoding::Pem) => {
            RsaPublicKey::from_pkcs1_pem(std::str::from_utf8(text)?)?
        }
        (KeyType::Pkcs8, KeyEncoding::Pem) => {
            RsaPublicKey::from_public_key_pem(std::str::from_utf8(text)?)?
        }
        (KeyType::Pkcs1, KeyEncoding::Der) => RsaPublicKey::from_pkcs1_der(text)?,
        (KeyType::Pkcs8, KeyEncoding::Der) => RsaPublicKey::from_public_key_der(text)?,
    };
    Ok(key)
}

fn get_pri_key(options: &crate::codecs::Options) -> anyhow::Result<RsaPrivateKey> {
    let text = options
        .get_text_raw("SK")
        .ok_or_else(|| anyhow::anyhow!("rsa: missing required option private key (-SK)"))?;
    let key = match get_key_type_and_encoding(options) {
        (KeyType::Pkcs1, KeyEncoding::Pem) => {
            RsaPrivateKey::from_pkcs1_pem(std::str::from_utf8(text)?)?
        }
        (KeyType::Pkcs8, KeyEncoding::Pem) => {
            RsaPrivateKey::from_pkcs8_pem(std::str::from_utf8(text)?)?
        }
        (KeyType::Pkcs1, KeyEncoding::Der) => RsaPrivateKey::from_pkcs1_der(text)?,
        (KeyType::Pkcs8, KeyEncoding::Der) => RsaPrivateKey::from_pkcs8_der(text)?,
    };
    Ok(key)
}

fn get_padding_scheme(padding: &str, hash: &str) -> anyhow::Result<OaepOrPkcs15> {
    let scheme = match padding {
        "oaep" => OaepOrPkcs15::Oaep(match hash {
            "sha1" => Oaep::new::<sha1::Sha1>(),
            "sha256" => Oaep::new::<sha2::Sha256>(),
            _ => anyhow::bail!("invalid hash type: {}", hash),
        }),
        "pkcs15" => OaepOrPkcs15::Pkcs15(Pkcs1v15Encrypt),

        _ => anyhow::bail!("invalid padding scheme: {}", padding),
    };
    Ok(scheme)
}

#[enum_delegate::implement(PaddingScheme,
trait PaddingScheme {
    fn decrypt<Rng: CryptoRngCore>(
        self,
        rng: Option<&mut Rng>,
        priv_key: &RsaPrivateKey,
        ciphertext: &[u8],
    ) -> rsa::Result<Vec<u8>>;
    fn encrypt<Rng: CryptoRngCore>(
        self,
        rng: &mut Rng,
        pub_key: &RsaPublicKey,
        msg: &[u8],
    ) -> rsa::Result<Vec<u8>>;
}
)]
enum OaepOrPkcs15 {
    Oaep(Oaep),
    Pkcs15(Pkcs1v15Encrypt),
}
