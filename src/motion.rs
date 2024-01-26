use crate::Result;

pub struct MotionBuffer{
    pub text: String,
    pub time: u32,
}


impl MotionBuffer {
    pub fn new() -> Self{
        MotionBuffer { text: String::new(), time: 0 }
    }

    // might have to be async for timming
    pub fn parse() -> Result<()> {
        Ok(())
    }
}

