#[derive(Clone, Debug)]
pub struct Command {
    pub(super) options: Vec<CommandOption>,
    pub(super) codecs: Vec<Codec>,
}

#[derive(Clone, Debug)]
pub(super) struct Codec {
    pub(super) name: String,
    pub(super) options: Vec<CommandOption>,
}

#[derive(Clone, Debug)]
pub(super) enum CommandOption {
    Switch(String),
    Value { name: String, text: Text },
}

#[derive(Clone, Debug)]
pub(super) enum Text {
    String(String),
    Codecs { input: String, codecs: Vec<Codec> },
}

impl Default for Text {
    fn default() -> Self {
        Text::String(String::new())
    }
}
