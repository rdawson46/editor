use tokio::{sync::mpsc, select};

// TODO: figure how to do leader
pub struct MotionBuffer {
    pub action: Option<String>,
    pub action_arg: Option<String>,
    pub number: Option<String>,
    pub motion: Option<String>,
}

impl Clone for MotionBuffer {
    fn clone(&self) -> Self {
        let mut new_mb = MotionBuffer::new();

        new_mb.action = self.action.clone();
        new_mb.action_arg = self.action_arg.clone();
        new_mb.number = self.number.clone();
        new_mb.motion = self.motion.clone();

        new_mb
    }
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
pub struct MotionHandler {
    // TODO: might need to make this a listener for KeyEvents
    pub listener: mpsc::UnboundedReceiver<char>, // listen for key strokes in normal mode
    pub clear: mpsc::UnboundedReceiver<bool>,
    pub motion_buffer: MotionBuffer, // used to parse motions
    pub output: mpsc::UnboundedSender<MotionBuffer>, // send out action when ready to use
}


impl MotionHandler {
    pub fn new(output: mpsc::UnboundedSender<MotionBuffer>) -> (Self, mpsc::UnboundedSender<char>, mpsc::UnboundedSender<bool>) {
        let (sender, listener) = mpsc::unbounded_channel::<char>();
        let (clear_sender, clear_listener) = mpsc::unbounded_channel::<bool>();
        let motion_b = MotionBuffer::new();

        let motion = MotionHandler {
            listener,
            clear: clear_listener,
            motion_buffer: motion_b,
            output
        };

        (motion, sender, clear_sender)
    }

    pub async fn listen(&mut self) {
        select! {
            recv = self.listener.recv() => {
                if let Some(c) = recv {
                    let c = c.clone();
                    let x = self.motion_buffer.push(c);

                    if let Some(_) = x {
                        let new_motion = self.motion_buffer.clone();
                        self.motion_buffer.clear();
                        let _res = self.send(new_motion);
                    }
                }
            }

            _ = self.clear.recv() => {
                self.motion_buffer.clear()
            }
        }
    }

    fn send(&mut self, a: MotionBuffer) -> Option<MotionBuffer> {
        let res = self.output.send(a);
        if let Ok(_) = res {
            return None;
        }
        Some(res.unwrap_err().0)
    }
}
