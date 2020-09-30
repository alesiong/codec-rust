pub struct Command {
    options: Vec<CommandOption>,
    pub(super) codecs: Vec<Codec>,
}

pub(super) struct Codec {
    pub(super) name: String,
    pub(super) options: Vec<CommandOption>,
}

pub(super) enum CommandOption {
    Switch(String),
    Value { name: String, text: Text },
}

pub(super) enum Text {
    String(String),
    Codecs { input: String, codecs: Vec<Codec> },
}

pub fn test_command() -> Command {
    Command {
        options: vec![],
        codecs: vec![Codec {
            name: "const".to_string(),
            options: vec![CommandOption::Value {
                name: "C".to_string(),
                text: Text::String("test-word".to_string()),
            }],
        }],
    }
}
