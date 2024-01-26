
struct Command {
    text: String,
    history: Vec<String>
}

impl Command {
    pub fn new() -> Self {
        Command { text: String::new(), history: vec![] }
    }
}
