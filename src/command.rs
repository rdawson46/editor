
pub struct Command {
    pub text: String,
    pub history: Vec<String>
}

impl Command {
    pub fn new() -> Self {
        Command { text: String::new(), history: vec![] }
    }
}
