use tokio::{sync::mpsc, select};

/* IDEA:
 * make the motion buffer a state machine
 *
 * need a new idea of how to process chars
 *  - need a way to generate an automatic table
 */

#[derive(Default, Clone, Copy)]
enum States {
    #[default] Start,
    Leader,
    NeedsParam,
    End,
}

struct StateMachine {
    state: States,
    input: String,
}

impl StateMachine {
    fn new() -> Self {
        StateMachine{
            state: States::default(),
            input: String::new(),
        }
    }

    // need to establish s
    fn recv(&mut self, c: char) -> States {
        let motions =  [':', 'j', 'k', 'h', 'l', 'i', 'a', 'w', 'b', 'e', '0', '$', 'I', 'A', 'O', 'o'];
        let needs_param = ['d', 'f'];
        let leader = ' ';

        match &self.state {
            States::Start => {
                if c.is_digit(10) {
                    self.input.push(c);
                    self.state = States::Start;
                } else if motions.contains(&c) {
                    self.input.push(c);
                    self.state = States::End;
                } else if needs_param.contains(&c) {
                    self.input.push(c);
                    self.state = States::NeedsParam;
                } else if leader == c {
                    self.input.push_str("<leader>");
                    self.state = States::Leader;
                }
            },
            States::NeedsParam => {
                self.input.push(c);
                self.state = States::End;
            },
            States::Leader => {
                // TODO: leader functions
            }
            States::End => self.state = States::Start,
        }

        self.state.clone()
    }

    fn fetch(&mut self) -> String {
        return self.input.clone();
    }

    fn refresh(&mut self) {
        self.state = States::default();
        self.input = String::new();
    }
}


pub struct MotionHandler {
    pub listener: mpsc::UnboundedReceiver<char>, // listen for key strokes in normal mode
    pub clear: mpsc::UnboundedReceiver<bool>,
    pub state_machine: StateMachine, // used to parse motions
    pub output: mpsc::UnboundedSender<String>, // send out action when ready to use
}

impl MotionHandler {
    pub fn new(output: mpsc::UnboundedSender<String>) -> (Self, mpsc::UnboundedSender<char>, mpsc::UnboundedSender<bool>) {
        let (sender, listener) = mpsc::unbounded_channel::<char>();
        let (clear_sender, clear_listener) = mpsc::unbounded_channel::<bool>();
        let state_m = StateMachine::new();

        let motion = MotionHandler {
            listener,
            clear: clear_listener,
            state_machine: state_m,
            output
        };

        (motion, sender, clear_sender)
    }

    pub async fn listen(&mut self) {
        select! {
            recv = self.listener.recv() => {
                if let Some(c) = recv {
                    let c = c.clone();
                    let x = self.state_machine.recv(c);

                    match x {
                        States::End => {
                            let finished_motion = self.state_machine.fetch();
                            self.state_machine.refresh();
                            let _res = self.send(finished_motion);
                        },
                        _ => {},
                    }

                }
            }

            _ = self.clear.recv() => {
                self.state_machine.refresh();
            }
        }
    }

    fn send(&mut self, a: String) -> Option<String> {
        let res = self.output.send(a);
        if let Ok(_) = res {
            return None;
        }
        Some(res.unwrap_err().0)
    }
}
