pub enum CommandKey {
    Save,
    Quit,
    Line(usize),
    SaveAndQuit,
    Logger,
    Send(String),
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
        self.text.clear();

        let ck: Option<CommandKey>;

        if let Ok(number) = command.parse::<usize>() {
            ck = Some(CommandKey::Line(number));
        } else {
            // TODO: parse commands to allow for args to be passed
            let args: Vec<&str> = command.split(' ').collect();
            //ck = match command.as_str() {
            ck = match *args.get(0).expect("") {
                "wq" | "x" => Some(CommandKey::SaveAndQuit),
                "q" => Some(CommandKey::Quit),
                "w" => Some(CommandKey::Save),
                "history" => Some(CommandKey::History),
                "logger" => Some(CommandKey::Logger),
                "send" => {
                    let mes = args[1..].join(" ");
                    Some(CommandKey::Send(mes.to_string()))
                },
                _ => None
            };
        }
        self.history.push(command);

        ck
    }
}
