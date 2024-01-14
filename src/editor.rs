use color_eyre::eyre::Result;
use std::path::{PathBuf, Path};
use std::fs::File;
use std::io::{BufReader, BufRead};

pub enum Mode{
    Insert, 
    Normal
}

pub struct Cursor{
    pub current: (u16, u16),
    pub possible: (u16, u16)
}

// TODO: fix movements to be safeer
    // implement stuff for possible
impl Cursor{
    fn new() -> Cursor{
        Cursor { current: (0,0), possible: (0,0) }
    }

    pub fn move_down(&mut self) {
        //self.current.0 += 1;
        self.current.1 = self.current.1.checked_add(1).unwrap_or(self.current.1);
    }

    pub fn move_up(&mut self) {
        //self.current.1 -= 1;
        self.current.1 = self.current.1.checked_sub(1).unwrap_or(self.current.1);
    }

    pub fn move_right(&mut self) {
        //self.current.0 += 1;
        self.current.0 = self.current.0.checked_add(1).unwrap_or(self.current.0);
    }

    pub fn move_left(&mut self) {
        //self.current.1 -= 1;
        self.current.0 = self.current.0.checked_sub(1).unwrap_or(self.current.0);
    }
}

pub struct Line{
    pub text: String,
    pub length: u16
}

// fields will be added later
pub struct Lines{
    pub lines: Vec<Line>
}

pub struct Editor{
    pub cursor: Cursor,
    pub lines: Lines,
    pub file: PathBuf,
    pub mode: Mode,
    pub should_quit: bool,
    pub cushion: u8,
}

impl Editor{
    pub fn new(path: &Path)-> Result<Editor> {
        // open file passed
        // load file to memory

        let file = File::open(&path);

        if let Ok(file) = file {
            let reader = BufReader::new(file);

            let mut lines: Vec<Line> = vec![];

            for line in reader.lines() {
                match line {
                    Ok(text) => {
                        let length: u16 = text.len().try_into().unwrap();
                        lines.push(Line { text, length });
                    }
                    Err(_) => {}
                }
            }
            
            let lines = Lines { lines };

            return Ok(Editor { cursor: Cursor::new(), lines, file: path.to_owned(), mode: Mode::Normal, should_quit: false, cushion: 0});
        }

        panic!("No file passed");
    }
}


