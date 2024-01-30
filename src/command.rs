pub enum CommandKey {
    Save,
    Quit,
    Line(usize),
    SaveAndQuit,
    History
}

pub struct Command {
    pub text: String,
    pub history: Vec<String>,
    pub ptr: usize
}

impl Command {
    pub fn new() -> Self {
        Command { text: String::new(), history: vec![], ptr: 0 }
    }

    pub fn confirm(&mut self) -> Option<CommandKey>{
        // make a command code enum in editor.rs
        // return an option of a code
        // handle after in update function
        let command = self.text.clone();
        self.text = String::new();

        // parse command
        // TODO: parse for numbers for line jump

        let ck: Option<CommandKey>;

        if let Ok(number) = command.parse::<usize>() {
            ck = Some(CommandKey::Line(number));
        } else {
            ck = match command.as_str() {
                "wq" | "x" => Some(CommandKey::SaveAndQuit),
                "q" => Some(CommandKey::Quit),
                "w" => Some(CommandKey::Save),
                "history" => Some(CommandKey::History),
                "number" => Some(CommandKey::Line(0)),
                _ => None
            };
        }
        self.history.push(command);

        ck
    }
}
