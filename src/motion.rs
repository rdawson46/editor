use tokio::sync::mpsc;

/* IDEA:
 * make the motion buffer a state machine
 *
 * need a new idea of how to process chars
 *  - need a way to generate an automatic table
 */

#[derive(Default, Clone, Copy, Debug, PartialEq)]
enum States {
    #[default] Start,
    Leader,
    NeedsParam,
    End,
}

struct StateMachine {
    state: States,
    queue: String,
    input: Vec<String>,
}

impl StateMachine {
    fn new() -> Self {
        StateMachine{
            state: States::default(),
            queue: String::new(),
            input: Vec::new(),
        }
    }

    fn push(&mut self, c: char) {
        if c.is_digit(10) {
            self.input.push(c.to_string());
        } else {
            if !self.queue.is_empty() {
                self.input.push(self.queue.clone());
                self.queue.clear();
            }

            self.input.push(String::from(c));
        }
    }

    fn push_str(&mut self, s: &str) {
        if !self.queue.is_empty() {
            self.input.push(self.queue.clone());
            self.queue.clear();
        }

        self.input.push(s.to_string());
    }

    // need to establish s
    fn recv(&mut self, c: char) -> States {
        let motions =  [':', 'j', 'k', 'h', 'l', 'i', 'a', 'w', 'b', 'e', '0', '$', 'I', 'A', 'O', 'o'];
        let needs_param = ['d', 'f'];
        let leader = ' ';

        match &self.state {
            States::Start => {
                if c.is_digit(10) {
                    self.push(c);
                    self.state = States::Start;
                } else if motions.contains(&c) {
                    self.push(c);
                    self.state = States::End;
                } else if needs_param.contains(&c) {
                    self.push(c);
                    self.state = States::NeedsParam;
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

    fn fetch(&self) -> Vec<String> {
        return self.input.clone();
    }

    fn to_string(&self) -> String {
        let mut s = String::new();

        for i in &self.input {
            s.push_str(i);
        }

        s.extend(self.queue.clone().chars());
        s
    }

    fn refresh(&mut self) {
        self.state = States::default();
        self.input.clear();
        self.queue.clear();
    }
}


pub struct MotionHandler {
    pub listener: mpsc::UnboundedReceiver<char>, // listen for key strokes in normal mode
    pub clear: mpsc::UnboundedReceiver<bool>,
    state_machine: StateMachine, // used to parse motions
    pub output: mpsc::UnboundedSender<Vec<String>>, // send out action when ready to use
}

impl MotionHandler {
    pub fn new() -> (Self, mpsc::UnboundedSender<char>, mpsc::UnboundedSender<bool>, mpsc::UnboundedReceiver<Vec<String>>) {
        let (sender, listener) = mpsc::unbounded_channel::<char>();
        let (clear_sender, clear_listener) = mpsc::unbounded_channel::<bool>();
        let state_m = StateMachine::new();
        let (output, input) = mpsc::unbounded_channel::<Vec<String>>(); // output from MotionHandler perspective

        let motion = MotionHandler {
            listener,
            clear: clear_listener,
            state_machine: state_m,
            output,
        };

        (motion, sender, clear_sender, input)
    }

    pub fn refresh(&mut self) {
        self.state_machine.refresh();
    }

    pub fn handle_char(&mut self, c: Option<char>) {
        if let Some(c) = c {
            let x = self.state_machine.recv(c);

            match x {
                States::End => {
                    let finished_motion = self.state_machine.fetch();
                    self.state_machine.refresh();
                    let _res = self.send(finished_motion);
                }
                _ => {},
            }
        }
    }

    fn send(&mut self, a: Vec<String>) -> Option<Vec<String>> {
        let res = self.output.send(a);
        if let Ok(_) = res {
            return None;
        }
        Some(res.unwrap_err().0)
    }

    pub fn get_text(&self) -> Option<String> {
        if self.state_machine.input.len() != 0 {
            return Some(self.state_machine.to_string());
        }
        None
    }
}

#[test]
fn test_motion() {
    use crate::MotionHandler;

    let (mut motion, _motion_sender, _clear_sender, _motion_buffer_listener) = MotionHandler::new();

    assert_eq!(motion.state_machine.state, States::Start);

    motion.handle_char(Some('d'));
    assert_eq!(motion.state_machine.input, vec!["d".to_string()]);
    assert_eq!(motion.state_machine.queue, "".to_string());
    assert_eq!(motion.state_machine.state, States::NeedsParam);

    motion.state_machine.refresh();
    assert!(motion.state_machine.input.is_empty());
    assert!(motion.state_machine.queue.is_empty());

    // should restart
    motion.handle_char(Some('j'));
    assert!(motion.state_machine.input.is_empty());
    assert!(motion.state_machine.queue.is_empty());
    assert_eq!(motion.state_machine.state, States::Start);
}
