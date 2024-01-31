pub struct MotionBuffer{
    pub text: String,
    pub time: u32, // might not be needed and only for leader key?
}


impl MotionBuffer {
    pub fn new() -> Self{
        MotionBuffer { text: String::new(), time: 0 }
    }

    pub fn push(&mut self, chr: char) {
        self.text.push(chr);
    }

    pub fn clear(&mut self) {
        self.text.clear();
    }
}

