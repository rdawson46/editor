use color_eyre::eyre::Result;
use std::path::{PathBuf, Path};
use std::fs::File;
use std::io::{BufReader, BufRead};
use std::usize;

pub enum Mode{
    Insert, 
    Normal
}

pub struct Cursor{
    pub current: (u16, u16),
    pub possible: (u16, u16)
}

impl Cursor{
    fn new() -> Cursor{
        Cursor { current: (0,0), possible: (0,0) }
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
    pub ptr: u16,
    pub size: (u16, u16)
}

// TODO: implement to possible cursor position
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

            return Ok(Editor { cursor: Cursor::new(), lines, file: path.to_owned(), mode: Mode::Normal, should_quit: false, cushion: 0, ptr: 0, size: (0, 0) });
        }

        panic!("No file passed");
    }

    // NOTE: display functions

    // TODO: return styled paragraph for the current mode to render
        // add return type -> import from ratatui 
    #[warn(dead_code)]
    pub fn mode_display(&self) {
        match &self.mode {
            Mode::Insert => todo!(),
            Mode::Normal => todo!(),
        }
    }

    // NOTE: mode change functions

    // TODO: incorporate changes for each mode
        // ex: change cursor?
    pub fn change_mode(&mut self, mode: Mode) {
        match mode {
            Mode::Insert => {
                self.mode = mode;
            },
            Mode::Normal => {
                self.mode = mode;
            },
        }
    }

    // NOTE: cursor movement methods

    pub fn move_down(&mut self) {
        // next logical y
        let y = self.cursor.current.1.checked_add(1).unwrap_or(self.cursor.current.1);

        if y > self.size.1 - 1 {
            if usize::from(self.size.1 + self.ptr) < self.lines.lines.len() {
                self.ptr += 1;
            }
            return;
        }

        // TODO: remove uselss bits
            // note: will need to still check if end of file

        // max lines in file
        let line_nums = self.lines.lines.len() - 1;

        // bottom of view
        // usize::from(self.ptr + self.size.1 - 1)
        //
        // if at bottom of view and not file, inc ptr
        
        // NOTE: this line will be useless with pointer movement
        let cap = std::cmp::min(line_nums, usize::from(self.ptr + self.size.1 - 1));
        let y = std::cmp::min(y, cap.try_into().unwrap());

        self.cursor.current.1 = y;

        let line_len = self.lines.lines.get(usize::from(self.cursor.current.1 + self.ptr)).unwrap().length;


        if line_len == 0 {
            self.cursor.current.0 = 0;
        } else {
            let x = std::cmp::max(self.cursor.current.0, self.cursor.possible.0);
            let x = std::cmp::min(x, line_len - 1);
            self.cursor.current.0 = x;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor.current.1 == 0 && self.ptr != 0 {
            self.ptr -= 1;
            return;
        }

        self.cursor.current.1 = self.cursor.current.1.checked_sub(1).unwrap_or(self.cursor.current.1);

        let line_len = self.lines.lines.get(usize::from(self.cursor.current.1 + self.ptr)).unwrap().length;

        if line_len == 0 {
            self.cursor.current.0 = 0;
        } else {
            let x = std::cmp::max(self.cursor.current.0, self.cursor.possible.0);
            let x = std::cmp::min(x, line_len - 1);
            self.cursor.current.0 = x;
        }
    }

    

    pub fn move_right(&mut self) {
        // self.cursor.current.0 = self.cursor.current.0.checked_add(1).unwrap_or(self.cursor.current.0);
        let line_len = self.lines.lines.get(usize::from(self.cursor.current.1 + self.ptr)).unwrap().length;
        if line_len == 0 {
            self.cursor.current.0 = 0;
        } else{
            let x = self.cursor.current.0.checked_add(1).unwrap_or(self.cursor.current.0);
            let x = std::cmp::min(x, line_len - 1);

            self.cursor.current.0 = x;
            self.cursor.possible.0 = x;
        }
    }

    pub fn move_left(&mut self) {
        let x = self.cursor.current.0.checked_sub(1).unwrap_or(self.cursor.current.0 + self.ptr);
        self.cursor.current.0 = x;
        self.cursor.possible.0 = x;
    }
}


