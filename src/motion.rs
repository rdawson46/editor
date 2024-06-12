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

pub struct InputMotion {
    pub sender: mpsc::UnboundedSender<char>,
}

impl InputMotion {
    fn new(sender: mpsc::UnboundedSender<char>) -> Self {
        InputMotion { sender }
    }

    pub async fn send(&mut self, x: char) -> Option<char> {
        let res = self.sender.send(x);
        if let Ok(_) = res {
            return None;
        }
        Some(res.unwrap_err().0)
    }
}

pub struct OutputMotion {
    pub listener: mpsc::UnboundedReceiver<char>,
    pub motion_buffer: MotionBuffer,
    pub output: mpsc::Sender<Action>,
}

impl OutputMotion {
    pub fn new(output: mpsc::Sender<Action>) -> (Self, InputMotion) {
        let (sender, listener) = mpsc::unbounded_channel::<char>();
        let motion_b = MotionBuffer::new();

        let motion = OutputMotion {
            listener,
            motion_buffer: motion_b,
            output
        };

        (motion, InputMotion::new(sender))
    }

    pub async fn start(&mut self) {
    }
}
