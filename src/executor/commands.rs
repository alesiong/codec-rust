#[derive(Clone)]
pub struct Command {
    pub(super) options: Vec<CommandOption>,
    pub(super) codecs: Vec<Codec>,
}

#[derive(Clone)]
pub(super) struct Codec {
    pub(super) name: String,
    pub(super) options: Vec<CommandOption>,
}

#[derive(Clone)]
pub(super) enum CommandOption {
    Switch(String),
    Value { name: String, text: Text },
}

#[derive(Clone)]
pub(super) enum Text {
    String(String),
    Codecs { input: String, codecs: Vec<Codec> },
}

#[allow(unused)]
pub fn test_command() -> Command {
    Command {
        options: vec![],
        codecs: vec![
            Codec {
                name: "repeat".to_string(),
                options: vec![
                    CommandOption::Value {
                        name: "T".to_string(),
                        text: Text::String("2".to_string()),
                    }
                ],
            }
            // Codec {
            //     name: "const".to_string(),
            //     options: vec![CommandOption::Value {
            //         name: "C".to_string(),
            //         text: Text::String("test-word".to_string()),
            //     }],
            // },
            // Codec {
            //     name: "id".to_string(),
            //     options: vec![],
            // },
        ],
    }
}
