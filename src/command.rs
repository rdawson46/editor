pub enum CommandKey {
    Save,
    Quit,
    Line(usize),
    SaveAndQuit,
    Logger,
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
        let command = self.text.clone();
        self.text = String::new();

        let ck: Option<CommandKey>;

        if let Ok(number) = command.parse::<usize>() {
            ck = Some(CommandKey::Line(number));
        } else {
            ck = match command.as_str() {
                "wq" | "x" => Some(CommandKey::SaveAndQuit),
                "q" => Some(CommandKey::Quit),
                "w" => Some(CommandKey::Save),
                "history" => Some(CommandKey::History),
                "logger" => Some(CommandKey::Logger),
                _ => None
            };
        }
        self.history.push(command);

        ck
    }
}
