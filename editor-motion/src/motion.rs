use tokio::sync::mpsc;
use crate::statemachine::{
    StateMachine,
    States
};

/* IDEA:
 * make the motion buffer a state machine
 *
 * need a new idea of how to process chars
 *  - need a way to generate an automatic table
 */



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

    motion.state_machine.refresh();
    assert!(motion.state_machine.input.is_empty());

    motion.handle_char(Some('j'));
    assert!(motion.state_machine.input.is_empty());
    assert_eq!(motion.state_machine.state, States::Start);

    motion.handle_char(Some('1'));
    motion.handle_char(Some('1'));
    motion.handle_char(Some('d'));
    assert_eq!(motion.state_machine.input, vec!["11".to_string(), "d".to_string()]);
    motion.handle_char(Some('j'));
    assert!(motion.state_machine.input.is_empty());
    assert_eq!(motion.state_machine.state, States::Start);
}
