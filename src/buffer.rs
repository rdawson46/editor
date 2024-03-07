use std::path::{Path, PathBuf};
use color_eyre::eyre::Result;
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
use std::fs::{File, read_dir};
use std::io::{BufReader, BufRead};
use crate::editor::{Cursor, Lines, Line, Mode};
use crossterm::{cursor, execute};
use crate::word::{
    find_word_end_forward,
    find_word_start_forward,
    find_word_start_backward
};

pub enum BufferType {
    Empty,
    Directory,
    File
}

pub struct Buffer {
    pub b_type: BufferType,
    pub lines: Lines,
    pub ptr_y: u16,
    pub ptr_x: u16,
    pub cursor: Cursor,
    pub file: Option<PathBuf>,
    pub mode: Mode
}

impl Buffer {
    pub fn new(path: &Path) -> Result<Buffer> {
        let btype: BufferType;
        let mut lines: Lines;
        let mut file_path: Option<PathBuf> = None;

        if path.exists() {
            if path.is_file() {
                let file = File::open(&path);

                if let Ok(file) = file {
                    let reader = BufReader::new(file);
                    btype = BufferType::File;

                    let mut file_lines: Vec<Line> = vec![];

                    for line in reader.lines() {
                        match line {
                            Ok(text) => {
                                let length: u16 = text.len().try_into().unwrap();
                                let boxed_text = Box::new(text);
                                file_lines.push( Line {
                                    text: boxed_text,
                                    length
                                });
                            },
                            Err(_) => {}
                        }
                    }

                    lines = Lines { lines: file_lines };
                    file_path = Some(path.to_owned());
                } else {
                    panic!("couldn't open file")
                }

            } else if path.is_dir() {
                btype = BufferType::Directory;
                lines = Lines { lines: vec![] };

                let reader = read_dir(path).unwrap();


                // TODO: sort entries and handle events for dirs
                // found error
                for path in reader {
                    let path = path.unwrap().file_name().into_string().unwrap();
                    let len = path.len();

                    let line: Line = Line { text: Box::new(path), length: len as u16 };
                    lines.lines.push(line);
                }

            } else {
                panic!("what did you do");
            }
        } else {
            btype = BufferType::Empty;
            lines = Lines { lines: vec![] }
        }

        return Ok(Buffer {
            b_type: btype,
            lines,
            ptr_x: 0,
            ptr_y: 0,
            cursor: Cursor::new(),
            file: file_path,
            mode: Mode::Normal
        });
    }

    // NOTE: mode change functions

    pub fn change_mode(&mut self, mode: Mode) {
        match mode {
            Mode::Insert => {
                execute!(std::io::stderr(), cursor::SetCursorStyle::BlinkingBar).unwrap();
                self.mode = mode;
            },
            Mode::Command => {
                self.mode = mode;
            },
            Mode::Normal => {
                // recalc cursor pos
                // get current pos, compare to line length

                let line_len = self.lines.lines.get(usize::from(self.cursor.current.1 + self.ptr_y)).unwrap().length;
                
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

    // functions for handling inputs for different types
    #[warn(dead_code)]
    pub fn insert_key_dir(&mut self, key: KeyEvent) {

    }

    pub fn insert_key_file(&mut self, key: KeyEvent, size: (u16, u16)) {
        match key.code {
            KeyCode::Char(value) => {
                if value == 'c' && key.modifiers == KeyModifiers::CONTROL {
                    self.change_mode(Mode::Normal);
                } else {
                    // get current line
                    let line_index = usize::from(self.ptr_y + self.cursor.current.1);

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
            KeyCode::Enter => {
                let curr_line = usize::from(self.ptr_y + self.cursor.current.1);

                let curr_index = usize::from(self.cursor.current.0);

                let new_str = self.lines.lines[curr_line].text.split_off(curr_index);

                let len: u16 = new_str.len() as u16;

                let new_line: Line = Line { text: Box::new(new_str), length: len };

                self.lines.lines.insert(curr_line + 1, new_line);
                self.move_down(size);
                self.cursor.current.0 = 0;
                self.cursor.possible.0 = 0;
            },
            KeyCode::Backspace => {
                let line_index = usize::from(self.ptr_y + self.cursor.current.1);

                let current_line = &mut self.lines.lines[line_index];
                let text_index = usize::from(self.cursor.current.0);

                if current_line.length != 0 && text_index > 0 {
                    current_line.length -= 1;
                    let text = &mut current_line.text;
                    text.remove(text_index.checked_sub(1).unwrap_or(0));
                    self.move_left();
                } else{
                    if line_index == 0 {
                        return;
                    }

                    self.move_up();
                    self.move_end_of_line();

                    let current_line = &self.lines.lines[line_index];
                    let text = current_line.text.clone();
                    self.lines.lines.remove(line_index);
                    let line_above = self.lines.lines.get_mut(line_index-1).unwrap();

                    line_above.text = Box::new(format!("{}{}", *line_above.text, &text));
                    line_above.length = line_above.text.len().try_into().unwrap();
                }
            },
            KeyCode::Tab => {},
            KeyCode::Esc => {
                self.change_mode(Mode::Normal);
            },
            _ => {}
        }
    }

    pub fn move_down(&mut self, size: (u16, u16)) {
        // next logical y
        let y = self.cursor.current.1.checked_add(1).unwrap_or(self.cursor.current.1);

        if y > size.1.checked_sub(1).unwrap_or(0) {
            if usize::from(size.1 + self.ptr_y) < self.lines.lines.len() {
                self.ptr_y += 1;
            }
            return;
        }

        // max lines in file
        let line_nums = self.lines.lines.len() - 1;

        
        // NOTE: this line will be useless with pointer movement
        let cap = std::cmp::min(line_nums, usize::from(self.ptr_y + size.1 - 1));
        let y = std::cmp::min(y, cap.try_into().unwrap());

        self.cursor.current.1 = y;

        let line_len = self.lines.lines.get(usize::from(self.cursor.current.1 + self.ptr_y)).unwrap().length;


        if line_len == 0 {
            self.cursor.current.0 = 0;
        } else {
            let x = std::cmp::max(self.cursor.current.0, self.cursor.possible.0);
            let x = std::cmp::min(x, line_len - 1);
            self.cursor.current.0 = x;
        }
    }

    pub fn move_up(&mut self) {
        if self.cursor.current.1 == 0 && self.ptr_y != 0 {
            self.ptr_y -= 1;
            return;
        }

        self.cursor.current.1 = self.cursor.current.1.checked_sub(1).unwrap_or(self.cursor.current.1);

        let line_len = self.lines.lines.get(usize::from(self.cursor.current.1 + self.ptr_y)).unwrap().length;

        if line_len == 0 {
            self.cursor.current.0 = 0;
        } else {
            let x = std::cmp::max(self.cursor.current.0, self.cursor.possible.0);
            let x = std::cmp::min(x, line_len - 1);
            self.cursor.current.0 = x;
        }
    }

    

    // TODO: cursor movement when in command mode
    pub fn move_right(&mut self) {
        // self.cursor.current.0 = self.cursor.current.0.checked_add(1).unwrap_or(self.cursor.current.0);
        let line_len = self.lines.lines.get(usize::from(self.cursor.current.1 + self.ptr_y)).unwrap().length;
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
                },
                Mode::Command => {
                    todo!()
                }
            }
        }
    }

    pub fn move_left(&mut self) {
        let x = self.cursor.current.0.checked_sub(1).unwrap_or(self.cursor.current.0 + self.ptr_y);
        self.cursor.current.0 = x;
        self.cursor.possible.0 = x;
    }

    pub fn move_end_of_line(&mut self) {
        let line_len = self.lines.lines.get(usize::from(self.cursor.current.1 + self.ptr_y)).unwrap().length;
        if line_len == 0 {
            self.cursor.current.0 = 0;
            return;
        }

        match &self.mode {
            Mode::Normal => {
                self.cursor.current.0 = line_len - 1;
                self.cursor.possible.0 = line_len - 1;
            },
            Mode::Insert => {
                self.cursor.current.0 = line_len;
                self.cursor.possible.0 = line_len;
            },
            Mode::Command => {}
        }
    }

    // TODO: implemet this function
    pub fn move_begin_of_line(&mut self){
        self.cursor.current.0 = 0;
        self.cursor.possible.0 = 0;
    }

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
}
