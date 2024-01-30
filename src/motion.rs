use crate::Result;

pub struct MotionBuffer{
    pub text: String,
    pub time: u32, // might not be needed and only for leader key?
}


impl MotionBuffer {
    pub fn new() -> Self{
        MotionBuffer { text: String::new(), time: 0 }
    }

    // might have to be async for timming
    #[warn(dead_code)]
    pub fn parse(&mut self) -> Result<()> {
        Ok(())
    }

    #[warn(dead_code)]
    pub fn push(&mut self, chr: char) {
        self.text.push(chr);
    }
}

