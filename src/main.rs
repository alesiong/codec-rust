mod codecs;
mod executor;

fn main() {
    let codecs = load_builtins();

    let args: Vec<_> = std::env::args().skip(1).collect();
    let mut tokenizer = executor::parser::Tokenizer::new(args);

    let commands = executor::parser::parse_command(&mut tokenizer).unwrap();

    executor::execute(commands, codecs).unwrap();
}

fn load_builtins() -> codecs::CodecMetaInfo {
    let mut meta_info = codecs::CodecMetaInfo::new();
    meta_info.register("id", Box::<codecs::builtins::IdCodecs>::default());
    meta_info.register("const", Box::<codecs::builtins::ConstCodecs>::default());
    meta_info.register("repeat", Box::<codecs::builtins::RepeatCodecs>::default());

    meta_info
}
