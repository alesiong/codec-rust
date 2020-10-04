use std::{
    io::{Read, Write},
    ops::DerefMut,
    sync::Arc,
};

use anyhow::Result;

use crate::{codecs, executor::commands};

#[allow(unused)]
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
                OPTION_NEW_LINE => command.codecs.push(commands::Codec {
                    name: "newline".to_string(),
                    options: vec![],
                }),
                _ => {
                    anyhow::bail!("unknown option: {}", name);
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
                OPTION_INPUT_FILE => {
                    let codec = commands::Codec {
                        name: "cat".to_string(),
                        options: vec![
                            commands::CommandOption::Switch("c".to_string()),
                            commands::CommandOption::Value {
                                name: "F".to_string(),
                                text: text.clone(),
                            },
                        ],
                    };

                    command.codecs.insert(0, codec);
                }
                OPTION_OUTPUT_FILE => {
                    let codec = commands::Codec {
                        name: "redirect".to_string(),
                        options: vec![commands::CommandOption::Value {
                            name: "O".to_string(),
                            text: text.clone(),
                        }],
                    };
                    command.codecs.insert(0, codec);
                }
                _ => {
                    anyhow::bail!("unknown option: {}", name);
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
        // TODO: (prof) PipeReader::BufRead::fill_buf, PipeWriter::Write::write use lots of tiny vec
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

    if options.get_switch("e") {
        mode = codecs::CodecMode::Encoding;
    }
    if options.get_switch("d") {
        mode = codecs::CodecMode::Decoding;
    }

    let c = codecs_info
        .lookup(&codec.name)
        .ok_or(anyhow::anyhow!("codec not found: {}", codec.name))?;
    c.run_codec(&mut input, mode, &options, &mut output)
}

fn make_codec_options(
    codec: &commands::Codec,
    codecs_info: Arc<codecs::CodecMetaInfo>,
) -> Result<codecs::Options> {
    let mut option = codecs::Options::new();

    for o in &codec.options {
        match o {
            commands::CommandOption::Switch(name) => {
                option.insert_switch(&name);
            }
            commands::CommandOption::Value { name, text } => match text {
                commands::Text::String(value) => {
                    option.insert_text(&name, value.as_bytes());
                }
                commands::Text::Codecs { input, codecs } => {
                    let mut buf = Vec::<u8>::new();

                    run_codecs(
                        Box::new(std::io::Cursor::new(input.clone())),
                        codecs.clone(),
                        Arc::clone(&codecs_info),
                        codecs::CodecMode::Encoding,
                        &mut buf,
                    )?;

                    option.insert_text(&name, &buf);
                }
            },
        }
    }

    Ok(option)
}
