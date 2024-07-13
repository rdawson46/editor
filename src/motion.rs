use tokio::sync::mpsc;

// TODO: figure how to do leader
pub struct MotionBuffer {
    pub action: Option<String>,
    pub action_arg: Option<String>,
    pub number: Option<String>,
    pub motion: Option<String>,
}

impl MotionBuffer {
    pub fn new() -> Self {
        MotionBuffer { action: None, action_arg: None, number: None, motion: None }
    }

    pub fn push(&mut self, chr: char) -> Option<u32> {
        // IDEA: trigger parsing when motion is hit
            // should I mark mode changers (i) as a motion for simplicity

        // TODO: determine how to use command_arg for f/t search, and o/O
        let motions =  [':', 'j', 'k', 'h', 'l', 'i', 'a', 'w', 'b', 'e', '0', '$', 'I', 'A', 'O', 'o'];
        let actions = ['d', 's', 'f'];

        if chr.is_digit(10) {
            if let Some(number) = &mut self.number {
                number.push(chr);
            } else {
                if chr == '0' {
                    self.motion = Some(String::from(chr));
                    return Some(0);
                }
                self.number = Some(String::from(chr));
            }
        } else if motions.contains(&chr) {
            if let Some(motion) = &mut self.motion {
                motion.push(chr);
            } else {
                self.motion = Some(String::from(chr));
            }
            return Some(0);
            
        } else if actions.contains(&chr) {
            if let Some(command) = &mut self.action {
                command.push(chr);
            } else {
                self.action = Some(String::from(chr));
            }
        }

        None
    }

    pub fn clear(&mut self) {
        self.action = None;
        self.number = None;
        self.motion = None;
    }
}


// WORK IN PROGRESS ==========
pub struct Action {
    action: Option<String>,
    action_arg: Option<String>,
    number: Option<String>,
    motion: Option<String>,
}

impl Action {
    fn new(motion_b: &mut MotionBuffer) -> Self {
        let action = Action {
            action: motion_b.action.clone(),
            action_arg: motion_b.action_arg.clone(),
            number: motion_b.number.clone(),
            motion: motion_b.motion.clone(),
        };
        motion_b.clear();
        action
    }
}

// DONT INCLUDE IN EDITOR, JUST GIVE EDITOR THE SENDER FOR LISTENER
// AND THE RECIEVER FROM OUTPUT
// PUT MOTIONHANDLER ON OTHER THREAD & JOINHANDLER
// or ...
pub struct MotionHandler {
    // TODO: might need to make this a listener for KeyEvents
    pub listener: mpsc::UnboundedReceiver<char>, // listen for key strokes in normal mode
    pub motion_buffer: MotionBuffer, // used to parse motions
    pub output: mpsc::UnboundedSender<Action>, // send out action when ready to use
}


impl MotionHandler {
    pub fn new(output: mpsc::UnboundedSender<Action>) -> (Self, mpsc::UnboundedSender<char>) {
        let (sender, listener) = mpsc::unbounded_channel::<char>();
        let motion_b = MotionBuffer::new();

        let motion = MotionHandler {
            listener,
            motion_buffer: motion_b,
            output
        };

        (motion, sender)
    }

    pub async fn listen(&mut self) {
        let recv = self.listener.recv().await;

        if let Some(c) = recv {
            let c = c.clone();
            let x = self.motion_buffer.push(c);

            if let Some(_) = x {
                let action = Action::new(&mut self.motion_buffer);
                let _res = self.send(action);
            }
        }
    }

    fn send(&mut self, a: Action) -> Option<Action> {
        let res = self.output.send(a);
        if let Ok(_) = res {
            return None;
        }
        Some(res.unwrap_err().0)
    }
}
