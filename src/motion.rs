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
