use color_eyre::eyre::Result;
use std::path::{PathBuf, Path};
use std::fs::{
    File,
    OpenOptions
};
use std::io::{
    BufReader,
    BufRead,
    BufWriter,
};
use std::io::prelude::*;
use std::usize;
use crossterm::{cursor, execute};
use crossterm::event::{
    KeyEvent,
    KeyCode,
    KeyModifiers
};
use ratatui::widgets::{
    Paragraph, Borders, Block
};
use crate::word::{
    find_word_end_forward,
    find_word_start_forward,
    find_word_start_backward
};


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
    pub text: Box<String>,
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

// TODO: implement saving

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
                        let boxed_text = Box::new(text);
                        lines.push(Line { text: boxed_text, length });
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

    pub fn mode_display(&self) -> Paragraph {
        match &self.mode {
            Mode::Insert => {
                Paragraph::new("-- Insert --").block(Block::default().borders(Borders::TOP))
            },
            Mode::Normal => {
                Paragraph::new("-- Normal --").block(Block::default().borders(Borders::TOP))
            },
        }
    }


    // NOTE: mode change functions

    // TODO: fix to adjust position of cursor when changing modes
    pub fn change_mode(&mut self, mode: Mode) {
        match mode {
            Mode::Insert => {
                execute!(std::io::stderr(), cursor::SetCursorStyle::BlinkingBar).unwrap();
                self.mode = mode;
            },
            Mode::Normal => {
                // recalc cursor pos
                // get current pos, compare to line length

                let line_len = self.lines.lines.get(usize::from(self.cursor.current.1 + self.ptr)).unwrap().length;
                
                if line_len == 0 {
                    self.cursor.current.0 = 0;
                } else {
                    let x = std::cmp::min(self.cursor.current.0, line_len - 1);
                    self.cursor.current.0 = x;
                    self.cursor.possible.0 = x;
                };

                execute!(std::io::stderr(), cursor::SetCursorStyle::SteadyBlock).unwrap();
                self.mode = mode;
            },
        }
    }

    // note: not specifically for inserting a key, but key handling in insert mode
    pub fn insert_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(value) => {
                if value == 'c' && key.modifiers == KeyModifiers::CONTROL {
                    self.change_mode(Mode::Normal);
                } else {
                    // get current line
                    let line_index = usize::from(self.ptr + self.cursor.current.1);

                    // WARN: doesn't work on index 0 for empty lines
                        // issue with cursor movement
                    let current_line = &mut self.lines.lines[line_index];
                    let text = &mut current_line.text;

                    let text_index = usize::from(self.cursor.current.0);

                    if current_line.length == 0 {
                        text.push(value);
                    } else {
                        text.insert(text_index, value);
                    }

                    current_line.length += 1;
                    self.move_right();
                }
            },
            KeyCode::Enter => {},
            KeyCode::Backspace => {
                let line_index = usize::from(self.ptr + self.cursor.current.1);

                let current_line = &mut self.lines.lines[line_index];
                let text_index = usize::from(self.cursor.current.0);

                // FIX: very bad, not cool, try again
                    // check if cursor is at eol
                    // remove from index sooner
                if current_line.length != 0 && text_index > 0 {
                    current_line.length -= 1;
                    let text = &mut current_line.text;
                    text.remove(text_index.checked_sub(1).unwrap_or(0));
                    self.move_left();
                } else{
                    // TODO: implement line removal
                }
            },
            KeyCode::Tab => {},
            KeyCode::Esc => {
                self.change_mode(Mode::Normal);
            },
            _ => {}
        }
    }

    // NOTE: cursor movement methods

    pub fn move_down(&mut self) {
        // next logical y
        let y = self.cursor.current.1.checked_add(1).unwrap_or(self.cursor.current.1);

        if y > self.size.1.checked_sub(1).unwrap_or(0) {
            if usize::from(self.size.1 + self.ptr) < self.lines.lines.len() {
                self.ptr += 1;
            }
            return;
        }

        // TODO: remove uselss bits

        // max lines in file
        let line_nums = self.lines.lines.len() - 1;

        
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

    

    // FIX: cursor movement when in insert mode
    pub fn move_right(&mut self) {
        // self.cursor.current.0 = self.cursor.current.0.checked_add(1).unwrap_or(self.cursor.current.0);
        let line_len = self.lines.lines.get(usize::from(self.cursor.current.1 + self.ptr)).unwrap().length;
        if line_len == 0 {
            self.cursor.current.0 = 0;
        } else{
            match &self.mode {
                Mode::Normal => {
                    let x = self.cursor.current.0.checked_add(1).unwrap_or(self.cursor.current.0);
                    let x = std::cmp::min(x, line_len - 1);

                    self.cursor.current.0 = x;
                    self.cursor.possible.0 = x;
                },
                Mode::Insert => {
                    let x = self.cursor.current.0.checked_add(1).unwrap_or(self.cursor.current.0);
                    let x = std::cmp::min(x, line_len);

                    self.cursor.current.0 = x;
                    self.cursor.possible.0 = x;
                }
            }
        }
    }

    pub fn move_left(&mut self) {
        let x = self.cursor.current.0.checked_sub(1).unwrap_or(self.cursor.current.0 + self.ptr);
        self.cursor.current.0 = x;
        self.cursor.possible.0 = x;
    }


    // NOTE: word movements

    // TODO: make work with going to next line
    pub fn move_next_word(&mut self) {
        // conversion
        let line = &self.lines.lines.get(usize::from(self.cursor.current.1)).unwrap().text;

        // get start col
        let start_col = usize::from(self.cursor.current.0);

        // find col
        let next = find_word_start_forward(line, start_col);

        // move cursor
        match next {
            Some(index) => {
                self.cursor.current.0 = index as u16;
                self.cursor.possible.0 = index as u16;
            },
            None => {}
        }
    }

    pub fn move_end_word(&mut self) {
        let line = &self.lines.lines.get(usize::from(self.cursor.current.1)).unwrap().text;
        let start_col = usize::from(self.cursor.current.0);
        let next = find_word_end_forward(line, start_col);
        match next {
            Some(index) => {
                self.cursor.current.0 = index as u16;
                self.cursor.possible.0 = index as u16;
            },
            None => {}
        }
    }

    pub fn move_back_word(&mut self) {
        let line = &self.lines.lines.get(usize::from(self.cursor.current.1)).unwrap().text;
        let start_col = usize::from(self.cursor.current.0);
        let next = find_word_start_backward(line, start_col);
        match next {
            Some(index) => {
                self.cursor.current.0 = index as u16;
                self.cursor.possible.0 = index as u16;
            },
            None => {}
        }
    }


    // NOTE: saving functions

    pub fn save(&mut self) {
        // TODO: steps idea
            // open file for writing
            // write all lines to the file buf saved in self
            
        // FIX: writes ^M to the begining of ever line
            // use bufwriter
        /*
         * NOTE: original idea

        let mut total_string = "".to_string();

        for line in self.lines.lines.iter() {
            total_string.push_str(&line.text);

            total_string.push('\n');
            total_string.push('\r');
        }

        let status = std::fs::write(&self.file, total_string.as_bytes());

        match status {
            Ok(_) => {},
            Err(_) => panic!("writing to file didn't work"),
        }
        */

        /*
         * NOTE: second idea, doesn't work as anticipated

        let file = OpenOptions::new()
            .write(true)
            .append(true)
            .open(&self.file)
            .expect("Couldn't open file when saving");

        let mut writer = BufWriter::new(file);

        for line in self.lines.lines.iter() {
            let mut temp_str = "".to_string();
            let text = &line.text;

            temp_str.push_str(text);
            writer.write_all(temp_str.as_bytes()).unwrap();
        }
        */
    }
}
