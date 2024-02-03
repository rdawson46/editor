// once motion is pressed, perform
    //  could possile use channels for this
pub struct MotionBuffer{
    pub command: Option<String>,
    pub number: Option<String>,
    pub motion: Option<String>,
    // will need these later? maybe in editor
    // motions: [char],
    // commands: [char],
}


impl MotionBuffer {
    pub fn new() -> Self{
        MotionBuffer { command: None, number: None, motion: None }
    }

    pub fn push(&mut self, chr: char) -> Option<u32> {
        // IDEA: trigger parsing when motion is hit
            // should I mark mode changers (i) as a motion for simplicity
        let motions =  [':', 'j', 'k', 'h', 'l', 'i', 'a', 'w', 'b', 'e', '0', '$', 'I', 'A'];
        let commands = ['d', 's'];

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
