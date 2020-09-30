use std::{
    collections::HashMap,
    io::{Read, Write},
    ops::Deref,
    ops::DerefMut,
};

use crate::{codecs, executor::commands};

type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;

pub fn execute(command: &commands::Command, codecs_info: &codecs::CodecMetaInfo) -> Result<()> {
    let global_mode = codecs::CodecMode::Encoding;
    run_codecs(
        &mut std::io::stdin(),
        &command.codecs,
        codecs_info,
        global_mode,
        &mut std::io::stdout(),
    )
}

enum OwnedOrBorrowed<'a, T>
where
    T: ?Sized + 'a,
{
    Borrowed(&'a mut T),
    Owned(Box<T>),
}

impl<T: ?Sized> Deref for OwnedOrBorrowed<'_, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        match self {
            OwnedOrBorrowed::Borrowed(borrowed) => borrowed,
            OwnedOrBorrowed::Owned(ref owned) => &owned,
        }
    }
}

impl<T: ?Sized> DerefMut for OwnedOrBorrowed<'_, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            OwnedOrBorrowed::Borrowed(borrowed) => borrowed,
            OwnedOrBorrowed::Owned(ref mut owned) => owned.deref_mut(),
        }
    }
}

fn run_codecs<R: Read + ?Sized, W: Write + ?Sized>(
    mut input: &mut R,
    codec_list: &[commands::Codec],
    codecs_info: &codecs::CodecMetaInfo,
    mode: codecs::CodecMode,
    output: &mut W,
) -> Result<()> {
    let mut previous_input = OwnedOrBorrowed::Borrowed(&mut input as &mut dyn Read);
    for c in codec_list {
        let (reader, mut writer) = pipe::pipe();
        run_codec(
            previous_input.deref_mut(),
            c,
            codecs_info,
            mode,
            &mut writer,
        )?;
        writer.flush()?;

        previous_input = OwnedOrBorrowed::Owned(Box::new(reader));
    }
    let _ = std::io::copy(previous_input.deref_mut(), output);
    Ok(())
}

fn run_codec<R: Read + ?Sized, W: Write + ?Sized>(
    mut input: &mut R,
    codec: &commands::Codec,
    codecs_info: &codecs::CodecMetaInfo,
    mut mode: codecs::CodecMode,
    mut output: &mut W,
) -> Result<()> {
    let options = make_codec_options(codec)?;

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

fn make_codec_options(codec: &commands::Codec) -> Result<HashMap<String, String>> {
    let mut option = HashMap::new();

    for o in &codec.options {
        match o {
            commands::CommandOption::Switch(name) => {
                option.insert(name.clone(), "*".to_string());
            }
            commands::CommandOption::Value { name, text } => match text {
                commands::Text::String(value) => {
                    option.insert(name.clone(), value.clone());
                }
                commands::Text::Codecs { input, codecs } => todo!(),
            },
        }
    }

    Ok(option)
}
