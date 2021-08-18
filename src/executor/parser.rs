use crate::executor::commands;

use anyhow::Result;

const OPENING_PARENTHESIS: &str = "[";
const CLOSING_PARENTHESIS: &str = "]";
const OPTION_PREFIX: &str = "-";

pub struct Tokenizer {
    look_next: Option<String>,
    text: Vec<String>,
    current_pos: usize,
    eof: bool,
}

impl Tokenizer {
    pub fn new(text: Vec<String>) -> Tokenizer {
        Tokenizer {
            text,
            look_next: None,
            current_pos: 0,
            eof: false,
        }
    }

    fn next(&mut self) -> Option<String> {
        if let Some(look_next) = self.look_next.take() {
            return Some(look_next);
        }

        if self.eof {
            return None;
        }

        let next = std::mem::take(&mut self.text[self.current_pos]);
        self.current_pos += 1;

        if next.starts_with(OPENING_PARENTHESIS) && next != OPENING_PARENTHESIS {
            self.look_next = Some(next[OPENING_PARENTHESIS.len()..].to_string());
            return Some(OPENING_PARENTHESIS.to_string());
        }

        if next.ends_with(CLOSING_PARENTHESIS) && next != CLOSING_PARENTHESIS {
            self.look_next = Some(CLOSING_PARENTHESIS.to_string());
            return Some(next[..next.len() - CLOSING_PARENTHESIS.len()].to_string());
        }

        Some(next)
    }

    fn peek(&mut self) -> Option<&str> {
        if self.look_next.is_some() {
            return self.look_next.as_deref();
        }
        if self.current_pos >= self.text.len() {
            self.eof = true;
        }

        if self.eof {
            return None;
        }

        let next = &self.text[self.current_pos];

        if next.starts_with(OPENING_PARENTHESIS) && next != OPENING_PARENTHESIS {
            return Some(OPENING_PARENTHESIS);
        }

        if next.ends_with(CLOSING_PARENTHESIS) && next != CLOSING_PARENTHESIS {
            return Some(&next[..next.len() - CLOSING_PARENTHESIS.len()]);
        }

        Some(next)
    }
}

pub fn parse_command(tokenizer: &mut Tokenizer) -> Result<commands::Command> {
    let options = parse_options(tokenizer)?;

    let mut codecs = vec![];

    while let Some(codec) = parse_codec(tokenizer)? {
        codecs.push(codec);
    }

    Ok(commands::Command { options, codecs })
}

fn is_special_token(token: &str) -> bool {
    token == OPENING_PARENTHESIS || token == CLOSING_PARENTHESIS
}

fn parse_codec(tokenizer: &mut Tokenizer) -> Result<Option<commands::Codec>> {
    let name = tokenizer.peek().unwrap_or_default();
    if name.is_empty() || is_special_token(name) {
        return Ok(None);
    }
    let name = name.to_owned();

    tokenizer.next();

    let options = parse_options(tokenizer)?;

    Ok(Some(commands::Codec { name, options }))
}

fn parse_options(tokenizer: &mut Tokenizer) -> Result<Vec<commands::CommandOption>> {
    let mut options = vec![];
    loop {
        let option_name = tokenizer.peek().unwrap_or_default();
        if !option_name.starts_with(OPTION_PREFIX) {
            break;
        }
        let option_name = option_name[OPTION_PREFIX.len()..].to_string();

        tokenizer.next();

        let option: commands::CommandOption;

        let first = option_name.chars().next().expect("empty option");
        if first.is_uppercase() {
            option = commands::CommandOption::Value {
                name: option_name,
                text: parse_text(tokenizer)?,
            }
        } else {
            option = commands::CommandOption::Switch(option_name);
        }

        options.push(option);
    }

    Ok(options)
}

fn parse_text(tokenizer: &mut Tokenizer) -> Result<commands::Text> {
    let str = tokenizer.peek();

    // TODO: allow empty option value
    if str.is_none() {
        anyhow::bail!("EOF when parsing");
    }

    let str = str.unwrap().to_string();

    tokenizer.next();

    let text: commands::Text;

    // TODO: escape parenthesis
    // TODO: allow empty sub-codecs syntax
    if str == OPENING_PARENTHESIS {
        let input = tokenizer.next().unwrap_or_default();
        let mut codecs = vec![];

        while let Some(codec) = parse_codec(tokenizer)? {
            codecs.push(codec);
        }

        let n = tokenizer.next().unwrap_or_default();
        if n != CLOSING_PARENTHESIS {
            anyhow::bail!("expect {}, found {}", CLOSING_PARENTHESIS, n);
        }

        text = commands::Text::Codecs { input, codecs };
    } else {
        text = commands::Text::String(str);
    }

    Ok(text)
}
