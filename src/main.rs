mod codecs;
mod executor;
mod utils;

use codecs::{builtins::*, meta::UsageMetaCodec, CodecMetaInfo};

fn main() {
    let codecs = load_builtins();
    CodecMetaInfo::set_instance(codecs);

    let args: Vec<_> = std::env::args().skip(1).collect();
    let mut tokenizer = executor::parser::Tokenizer::new(args);

    // TODO: eliminate unwrap

    let commands = executor::parser::parse_command(&mut tokenizer).unwrap_or_else(|err| {
        eprintln!("Error when parsing command: {}", err);
        std::process::exit(1)
    });

    executor::execute(commands).unwrap_or_else(|err| {
        eprintln!("Error in executing: {}", err);
        std::process::exit(1)
    });
}

fn load_builtins() -> codecs::CodecMetaInfo {
    let mut meta_info = codecs::CodecMetaInfo::new();
    meta_info.register_codec::<IdCodecs>("id");
    meta_info.register_codec::<ConstCodecs>("const");
    meta_info.register_codec::<RepeatCodecs>("repeat");
    meta_info.register_codec::<AppendCodecs>("append");
    meta_info.register_codec::<NewLineCodecs>("newline");
    meta_info.register_codec::<CatCodecs>("cat");
    meta_info.register_codec::<DropCodecs>("drop");
    meta_info.register_codec::<TakeCodecs>("take");
    meta_info.register_codec::<TeeCodecs>("tee");
    meta_info.register_codec::<SinkCodecs>("sink");
    meta_info.register_codec::<RedirectCodecs>("redirect");
    meta_info.register_codec::<Base64Codec>("base64");
    meta_info.register_codec::<HexCodec>("hex");
    meta_info.register("aes-cbc", AesCodec::new_cbc());
    meta_info.register("aes-ecb", AesCodec::new_ecb());
    meta_info.register("sm4-cbc", Sm4Codec::new_cbc());
    meta_info.register("sm4-ecb", Sm4Codec::new_ecb());
    meta_info.register("md5", HashCodec::new_md5());
    meta_info.register("sha256", HashCodec::new_sha256());
    meta_info.register("sm3", HashCodec::new_sm3());
    meta_info.register_codec::<UrlCodec>("url");
    meta_info.register_codec::<ZlibCodec>("zlib");
    meta_info.register_codec::<EscapeCodec>("escape");
    meta_info.register_codec::<RsaCryptCodec>("rsa-crypt");

    meta_info.register_meta("usage", UsageMetaCodec::default());
    meta_info
}
