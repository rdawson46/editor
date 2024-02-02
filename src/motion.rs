// once motion is pressed, perform
pub struct MotionBuffer{
    pub command: char,
    pub number: char,
    pub motion: char,
    // will need these later?
    // motions: [char],
    // commands: [char],
}


impl MotionBuffer {
    pub fn new() -> Self{
        MotionBuffer { command: '\0', number: '\0', motion: '\0' }
    }

    pub fn push(&mut self, chr: char) {
        let motions = ['a'];
        let commands = ['a'];

        if chr.is_digit(10) {
            self.number = chr;
        } else if motions.contains(&chr) {
            self.motion = chr;
        } else if commands.contains(&chr) {
            self.command = chr;
        }
    }

    pub fn clear(&mut self) {
        self.command = '\0';
        self.number = '\0';
        self.motion = '\0';
    }
}

