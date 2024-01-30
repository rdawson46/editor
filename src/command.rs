// IDEA: command enum
// since editor doesn't want to be passed to confirm function

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

    pub fn confirm(&mut self) {
        // make a command code enum in editor.rs
        // return an option of a code
        // handle after in update function
        let command = self.text.clone();
        self.text = String::new();

        // parse command
        match command.as_str() {
            "wq" => {},
            _ => {}
        }
        self.history.push(command);
    }
}
