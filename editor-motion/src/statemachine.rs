use std::collections::HashMap;

macro_rules! hashmap {
    (@single $($x:tt)*) => (());
    (@count $($rest:expr),*) => (<[()]>::len(&[$(hashmap!(@single $rest)),*]));
    ($($key:expr => $value:expr,)+) => { hashmap!($($key => $value),+) };
    ($($key:expr => $value:expr),*) => {
        {
            let _cap = hashmap!(@count $($key),*);
            let mut _map = ::std::collections::HashMap::with_capacity(_cap);
            $(
                let _ = _map.insert($key, $value);
            )*
            _map
        }
    };
}

#[derive(Default, Clone, Copy, Debug, PartialEq)]
pub enum States {
    #[default] Start,
    Leader,
    NeedsParam,
    End,
}

pub enum FunctionType {
    NeedsParam,
    Final
}

pub struct StateMachine {
    pub state: States,
    queue: String,
    pub input: Vec<String>,
    map: HashMap<String, FunctionType>,
}


// TODO: add functions from state machine
//
// reset to start from motions and 'functions'
// i.e. 'j', 'gg', 'G'
impl StateMachine {
    pub fn new() -> Self {
        StateMachine{
            state: States::default(),
            queue: String::new(),
            input: Vec::new(),
            map: hashmap! {
                ":".to_string() => FunctionType::Final,
                "j".to_string() => FunctionType::Final,
                "k".to_string() => FunctionType::Final,
                "h".to_string() => FunctionType::Final,
                "l".to_string() => FunctionType::Final,
                "i".to_string() => FunctionType::Final,
                "a".to_string() => FunctionType::Final,
                "w".to_string() => FunctionType::Final,
                "b".to_string() => FunctionType::Final,
                "e".to_string() => FunctionType::Final,
                "0".to_string() => FunctionType::Final,
                "$".to_string() => FunctionType::Final,
                "I".to_string() => FunctionType::Final,
                "A".to_string() => FunctionType::Final,
                "0".to_string() => FunctionType::Final,
                "o".to_string() => FunctionType::Final,

                "d".to_string() => FunctionType::NeedsParam,
                "f".to_string() => FunctionType::NeedsParam,
                "g".to_string() => FunctionType::NeedsParam,
            },
        }
    }

    // TODO: impl this function
    pub fn push(&mut self, c: char) {
        if c.is_digit(10) {
            self.queue.push(c);
        } else {
            if !self.queue.is_empty() {
                self.input.push(self.queue.clone());
                self.queue.clear();
            }

            self.input.push(String::from(c));
        }
    }

    // TODO: impl this function
    pub fn push_str(&mut self, s: &str) {
        if !self.queue.is_empty() {
            self.input.push(self.queue.clone());
            self.queue.clear();
        }

        self.input.push(s.to_string());
    }

    // need to establish s
    pub fn recv(&mut self, c: char) -> States {
        let leader = ' ';
        match &self.state {
            States::Start => {
                if c.is_digit(10) {
                    self.push(c);
                    self.state = States::Start;
                } else if let Some(t) = &self.map.get(&c.clone().to_string()){
                    match t {
                        FunctionType::NeedsParam => {
                            // look into how this worked before
                            self.push(c);
                            self.state = States::NeedsParam;
                        },

                        FunctionType::Final => {
                            self.push(c);
                            self.state = States::End;
                        },
                    }
                } else if leader == c {
                    self.push_str("<leader>");
                    self.state = States::Leader;
                }
            },
            States::NeedsParam => {
                if c.is_digit(10) {
                    self.push(c);
                    self.state = States::NeedsParam;
                } else {
                    self.push(c);
                    self.state = States::End;
                }
            },
            States::Leader => {
                // TODO: leader functions
            }
            States::End => self.state = States::Start,
        }

        self.state.clone()
    }

    pub fn fetch(&self) -> Vec<String> {
        return self.input.clone();
    }

    pub fn to_string(&self) -> String {
        let mut s = String::new();

        for i in &self.input {
            s.push_str(i);
        }

        s.extend(self.queue.clone().chars());
        s
    }

    pub fn refresh(&mut self) {
        self.state = States::default();
        self.input.clear();
        self.queue.clear();
    }
}

#[test]
fn test_motion() {
    let mut sm = StateMachine::new();

    sm.recv('d');
    assert_eq!(sm.input, vec!["d".to_string()]);
    assert_eq!(sm.state, States::NeedsParam);

    sm.refresh();
    assert!(sm.queue.is_empty());
}
