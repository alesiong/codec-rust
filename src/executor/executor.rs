use std::{
    io::{Read, Write},
    ops::DerefMut,
};

use anyhow::Result;

use crate::{
    codecs::{self, CodecMetaInfo},
    executor::commands,
};

#[allow(unused)]
const OPTION_ENCODING: &str = "e";
const OPTION_DECODING: &str = "d";
const OPTION_NEW_LINE: &str = "n";
const OPTION_INPUT_STRING: &str = "I";
const OPTION_INPUT_FILE: &str = "F";
const OPTION_OUTPUT_FILE: &str = "O";
const OPTION_HELP: &str = "h";

pub fn execute(mut command: commands::Command) -> Result<()> {
    let mut global_mode = codecs::CodecMode::Encoding;
    for o in &command.options {
        match o {
            commands::CommandOption::Switch(name) => match name.as_ref() {
                OPTION_ENCODING => global_mode = codecs::CodecMode::Encoding,
                OPTION_DECODING => global_mode = codecs::CodecMode::Decoding,
                OPTION_NEW_LINE => command.codecs.push(commands::Codec {
                    name: "newline".to_string(),
                    options: vec![],
                }),
                OPTION_HELP => {
                    let codec = commands::Codec {
                        name: "usage".to_string(),
                        options: vec![],
                    };
                    command.codecs = vec![codec];
                }
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
                    command.codecs.push(codec);
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
        global_mode,
        &mut std::io::stdout(),
    )
}

fn run_codecs<R: 'static + Read + ?Sized + Send, W: Write + ?Sized>(
    input: Box<R>,
    codec_list: Vec<commands::Codec>,
    mode: codecs::CodecMode,
    output: &mut W,
) -> Result<()> {
    let mut previous_input = Box::new(input) as Box<dyn Read + Send>;

    for c in codec_list {
        // TODO: (prof) PipeReader::BufRead::fill_buf, PipeWriter::Write::write use lots of tiny vec
        let (reader, mut writer) = pipe::pipe();

        std::thread::Builder::new()
            .name(c.name.clone())
            .spawn(move || {
                (|| -> Result<()> {
                    run_codec(&mut previous_input, &c, mode, &mut writer)?;
                    writer.flush()?;
                    Ok(())
                })()
                    .unwrap_or_else(|err| {
                        if let Some(io_err) = err.downcast_ref::<std::io::Error>() {
                            if io_err.kind() == std::io::ErrorKind::BrokenPipe {
                                return;
                            }
                        }
                        eprintln!("Error when executing codec {}: {}", c.name, err);
                        std::process::exit(1)
                    });
            })?;

        previous_input = Box::new(reader);
    }
    let _ = std::io::copy(previous_input.deref_mut(), output);
    Ok(())
}

fn run_codec<R: Read + ?Sized, W: Write + ?Sized>(
    mut input: &mut R,
    codec: &commands::Codec,
    mut mode: codecs::CodecMode,
    mut output: &mut W,
) -> Result<()> {
    let codecs_info = CodecMetaInfo::instance();
    let options = make_codec_options(codec)?;

    if options.get_switch("e") {
        mode = codecs::CodecMode::Encoding;
    }
    if options.get_switch("d") {
        mode = codecs::CodecMode::Decoding;
    }

    let c = codecs_info
        .lookup(&codec.name)
        .ok_or_else(|| anyhow::anyhow!("codec not found: {}", codec.name))?;
    c.run_codec(&mut input, mode, &options, &mut output)
}

fn make_codec_options(codec: &commands::Codec) -> Result<codecs::Options> {
    let mut option = codecs::Options::new();

    for o in &codec.options {
        match o {
            commands::CommandOption::Switch(name) => {
                option.insert_switch(name);
            }
            commands::CommandOption::Value { name, text } => match text {
                commands::Text::String(value) => {
                    option.insert_text(name, value.as_bytes());
                }
                commands::Text::Codecs { input, codecs } => {
                    let mut buf = Vec::<u8>::new();

                    run_codecs(
                        Box::new(std::io::Cursor::new(input.clone())),
                        codecs.clone(),
                        codecs::CodecMode::Encoding,
                        &mut buf,
                    )?;

                    option.insert_text(name, &buf);
                }
            },
        }
    }

    Ok(option)
}
