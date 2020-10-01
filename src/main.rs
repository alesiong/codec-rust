mod codecs;
mod executor;
mod utils;

use codecs::builtins::*;

fn main() {
    let codecs = load_builtins();

    let args: Vec<_> = std::env::args().skip(1).collect();
    let mut tokenizer = executor::parser::Tokenizer::new(args);

    // TODO: eliminate unwrap

    let commands = executor::parser::parse_command(&mut tokenizer).unwrap();

    executor::execute(commands, codecs).unwrap();
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

    meta_info
}
