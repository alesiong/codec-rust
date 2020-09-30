use std::{
    collections::HashMap,
    io::{Read, Write},
    ops::DerefMut,
    sync::Arc,
};

use crate::{codecs, executor::commands};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

const OPTION_ENCODING: &str = "e";
const OPTION_DECODING: &str = "d";
const OPTION_NEW_LINE: &str = "n";
const OPTION_INPUT_STRING: &str = "I";
const OPTION_INPUT_FILE: &str = "F";
const OPTION_OUTPUT_FILE: &str = "O";
const OPTION_HELP: &str = "h";

pub fn execute(mut command: commands::Command, codecs_info: codecs::CodecMetaInfo) -> Result<()> {
    let mut global_mode = codecs::CodecMode::Encoding;
    for o in &command.options {
        match o {
            commands::CommandOption::Switch(name) => match name.as_ref() {
                OPTION_DECODING => global_mode = codecs::CodecMode::Decoding,
                _ => {
                    return Err(format!("unknown option: {}", name).into());
                }
            },
            commands::CommandOption::Value { name, text } => match name.as_ref() {
                OPTION_INPUT_STRING => {
                    let codec = commands::Codec {
                        name: "const".to_string(),
                        options: vec![commands::CommandOption::Value {
                            name: "C".to_string(),
                            text: text.clone(),
                        }],
                    };

                    command.codecs.insert(0, codec);
                }
                _ => {
                    return Err(format!("unknown option: {}", name).into());
                }
            },
        }
    }

    run_codecs(
        Box::new(std::io::stdin()),
        command.codecs,
        Arc::new(codecs_info),
        global_mode,
        &mut std::io::stdout(),
    )
}

fn run_codecs<R: 'static + Read + ?Sized + Send, W: Write + ?Sized>(
    input: Box<R>,
    codec_list: Vec<commands::Codec>,
    codecs_info: Arc<codecs::CodecMetaInfo>,
    mode: codecs::CodecMode,
    output: &mut W,
) -> Result<()> {
    let mut previous_input = Box::new(input) as Box<dyn Read + Send>;

    for c in codec_list {
        let (reader, mut writer) = pipe::pipe();
        let codecs_info = Arc::clone(&codecs_info);

        std::thread::Builder::new()
            .name(c.name.clone())
            .spawn(move || {
                run_codec(&mut previous_input, &c, codecs_info, mode, &mut writer).unwrap();
                writer.flush().unwrap();
            })?;

        previous_input = Box::new(reader);
    }
    let _ = std::io::copy(previous_input.deref_mut(), output);
    Ok(())
}

fn run_codec<R: Read + ?Sized, W: Write + ?Sized>(
    mut input: &mut R,
    codec: &commands::Codec,
    codecs_info: Arc<codecs::CodecMetaInfo>,
    mut mode: codecs::CodecMode,
    mut output: &mut W,
) -> Result<()> {
    let options = make_codec_options(codec, Arc::clone(&codecs_info))?;

    if options.get("e").is_some() {
        mode = codecs::CodecMode::Encoding;
    }
    if options.get("d").is_some() {
        mode = codecs::CodecMode::Decoding;
    }

    let c = codecs_info
        .lookup(&codec.name)
        .ok_or(Box::<dyn std::error::Error>::from(format!(
            "codec not found: {}",
            codec.name
        )))?;
    c.run_codec(&mut input, mode, &options, &mut output)
}

fn make_codec_options(
    codec: &commands::Codec,
    codecs_info: Arc<codecs::CodecMetaInfo>,
) -> Result<HashMap<String, String>> {
    let mut option = HashMap::new();

    for o in &codec.options {
        match o {
            commands::CommandOption::Switch(name) => {
                option.insert(name.clone(), "*".to_string()); // TODO: eliminate hard coding
            }
            commands::CommandOption::Value { name, text } => match text {
                commands::Text::String(value) => {
                    option.insert(name.clone(), value.clone());
                }
                commands::Text::Codecs { input, codecs } => {
                    let input = bytebuffer::ByteBuffer::from_bytes(input.as_bytes());
                    let mut buf = bytebuffer::ByteBuffer::new();

                    run_codecs(
                        Box::new(input),
                        codecs.clone(),
                        Arc::clone(&codecs_info),
                        codecs::CodecMode::Encoding,
                        &mut buf,
                    )?;

                    option.insert(name.clone(), buf.to_string());
                }
            },
        }
    }

    Ok(option)
}
