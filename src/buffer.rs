use std::path::{Path, PathBuf};
use color_eyre::eyre::Result;
use std::fs::File;
use std::io::{BufReader, BufRead};

enum BufferType {
    Empty,
    Directory,
    File
}

pub struct Line{
    pub text: Box<String>,
    pub length: u16
}

// fields will be added later
pub struct Lines{
    pub lines: Vec<Line>
}

pub struct Buffer {
   b_type: BufferType,
   pub path: Option<PathBuf>,
   pub lines: Lines
}

impl Buffer {
    pub fn new(path: &Path) -> Result<Buffer> {
        // TODO: handle unkown paths better
        if path.exists() {
            if path.is_file() {

            } else if path.is_dir() {

            } else {
                panic!("I don't know what happened, but it's bad'");
            }
        } else {

        }

        todo!("not done");
    }
}
