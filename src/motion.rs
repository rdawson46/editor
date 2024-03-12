// once motion is pressed, perform
    //  could possile use channels for this
// TODO: figure how to do leader
pub struct MotionBuffer{
    pub command: Option<String>,
    pub command_arg: Option<String>,
    pub number: Option<String>,
    pub motion: Option<String>,
}


impl MotionBuffer {
    pub fn new() -> Self{
        MotionBuffer { command: None, command_arg: None, number: None, motion: None }
    }

    pub fn push(&mut self, chr: char) -> Option<u32> {
        // IDEA: trigger parsing when motion is hit
            // should I mark mode changers (i) as a motion for simplicity

        // TODO: determine how to use command_arg for f/t search, and o/O
        let motions =  [':', 'j', 'k', 'h', 'l', 'i', 'a', 'w', 'b', 'e', '0', '$', 'I', 'A', 'O', 'o'];
        let commands = ['d', 's', 'f'];

        if chr.is_digit(10) {
            // self.number.push(chr);

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
            
        } else if commands.contains(&chr) {
            if let Some(command) = &mut self.command {
                command.push(chr);
            } else {
                self.command = Some(String::from(chr));
            }
        }

        None
    }

    pub fn clear(&mut self) {
        self.command = None;
        self.number = None;
        self.motion = None;
    }
}
