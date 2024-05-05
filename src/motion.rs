// TODO: figure how to do leader
pub struct MotionBuffer {
    pub action: Option<String>,
    pub action_arg: Option<String>,
    pub number: Option<String>,
    pub motion: Option<String>,
}


/*
 * IDEAS:
 *
 *  have thread for parsing, when char pushed send to that thread and process
 *  maybe add thread to editor for listening for readiness
 *    - will complicate how to know when this is done and an action is ready to perform
 *        - maybe just have a thread running on the editor that will handle the action on a
 *          signal
 *
 * issue: having a thread open on the editor will create some borrow checker issues

// workup for concept 1
struct Signal {}

struct Concept{
    chars: Vec<char>,
    sender: Option<mpsc::Sender<Signal>>,
}

impl Concept {
    fn new() -> (Self, mpsc::Receiver<Signal>) {
        let (sender, receiver) = mpsc::channel();
        (Concept { chars: vec![], sender: Some(sender) }, receiver)
    }

    fn needs_args(c: &char) -> bool {
        let requires_args = [];

        requires_args.contains(c)
    }

    fn push(&mut self, c: char){
        // check if char needs args (another char)
        if Self::needs_args(&c) {
            self.chars.push(c);
        } else {
            // create signal and send
            let signal = Signal{};

            match &self.sender {
                Some(sender) => {
                    sender.send(signal).unwrap();
                },
                _ => {},
            }
        }
    }

    fn clear(&mut self) {
        self.chars.clear();
    }
}
*/

// single threaded concept
// TODO: might have to change types in the options
    // like motionbuffer to all String
struct Signal {
    motion: Option<char>,
    action: Option<char>,
    count: Option<u32>,
}

impl Signal {
    fn new() -> Self {
        Signal { motion: None, action: None, count: None }
    }
}

struct Concept{
    chars: Vec<char>,
}

impl Concept {
    fn new() -> Self {
        Concept { chars: vec![] }
    }

    fn needs_args(c: &char) -> bool {
        let requires_args = [];

        requires_args.contains(c)
    }

    // rework this
    fn push(&mut self, c: char) -> Option<Signal> {
        // check if char needs args (another char)
        if Self::needs_args(&c) {
            self.chars.push(c);
        } else {
            // create signal and return in option
            return Some(self.parse());
        }

        None
    }

    fn parse(&mut self) -> Signal {
        let mut signal = Signal::new();
        let motions =  [':', 'j', 'k', 'h', 'l', 'i', 'a', 'w', 'b', 'e', '0', '$', 'I', 'A', 'O', 'o'];
        let actions = ['d', 's', 'f'];

        for c in self.chars.iter() {
            if c.is_digit(10) {
                // TODO: figure out how to make this part work

            } else if motions.contains(&c) {
                signal.motion = Some(c.clone());
            } else if actions.contains(&c) {
                signal.action = Some(c.clone());
            }
        }

        signal
    }

    fn clear(&mut self) {
        self.chars.clear();
    }
}


impl MotionBuffer {
    pub fn new() -> Self {
        MotionBuffer { action: None, action_arg: None, number: None, motion: None }
    }

    /*
     * IDEA:
     * remove action vs motion split
     * TODO: use all the same, just determine if a char requires an arg to run
     *
     * take in char, determine if it needs a 'param'
     * if not, run
    */
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
