mod codecs;
mod executor;

fn main() {
    let codecs = load_builtins();

    let commands = executor::commands::test_command();

    executor::execute(commands, codecs).unwrap();
}

fn load_builtins() -> codecs::CodecMetaInfo {
    let mut meta_info = codecs::CodecMetaInfo::new();
    meta_info.register("id", Box::<codecs::builtins::IdCodecs>::default());
    meta_info.register("const", Box::<codecs::builtins::ConstCodecs>::default());
    meta_info.register("repeat", Box::<codecs::builtins::RepeatCodecs>::default());

    meta_info
}
