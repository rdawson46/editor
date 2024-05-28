use std::path::{Path, PathBuf};
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::fs::{File, read_dir};
use std::io::{BufReader, BufRead};
use crate::editor::{Cursor, Mode};
use crossterm::{cursor, execute};
use crate::word::{
    find_word_end_forward,
    find_word_start_forward,
    find_word_start_backward
};
use ropey::{
    Rope,
    RopeSlice,
    RopeBuilder,
};

#[derive(PartialEq)]
pub enum BufferType {
    Empty,
    Directory,
    File
}

// who put this in a box
pub struct Line{
    pub text: String,
    pub length: u16,
}

// fields will be added later
pub struct Lines{
    pub lines: Vec<Line>,
    pub rope: Rope,
}

// TODO: might need to add a variable for pathing
// idea: replace file with path
pub struct Buffer {
    pub buffer_type: BufferType,
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
        let mut rb = RopeBuilder::new();
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
                                file_lines.push( Line {
                                    text: text.clone(),
                                    length
                                });

                                rb.append(&text);
                            },
                            Err(_) => {}
                        }
                    }

                    // TODO: impl ropes
                    lines = Lines { lines: file_lines, rope: rb.finish() };
                    file_path = Some(path.to_owned());
                } else {
                    // FIX: remove this panic
                    panic!("couldn't open file")
                }

            } else if path.is_dir() {
                btype = BufferType::Directory;
                // TODO: impl ropes
                lines = Lines { lines: vec![], rope: Rope::new() };

                let self_dot = String::from(".");
                let parent_dot = String::from("..");

                let line: Line = Line { text: self_dot.clone(), length: self_dot.len() as u16 };
                lines.lines.push(line);
                lines.rope.append(".\n".into());

                let line: Line = Line { text: parent_dot.clone(), length: parent_dot.len() as u16 };
                lines.lines.push(line);
                lines.rope.append("..\n".into());


                let reader = read_dir(path).unwrap();

                for path in reader {
                    let path = path.unwrap().file_name().into_string().unwrap();
                    let len = path.len();

                    let line: Line = Line { text: path.clone(), length: len as u16 };
                    lines.lines.push(line);
                    lines.rope.append(path.into());
                }
            } else {
                panic!("no thank you");
            }
        } else {
            btype = BufferType::Empty;
            // TODO: impl ropes
            lines = Lines { lines: vec![], rope: Rope::new() }
        }

        return Ok(Buffer {
            buffer_type: btype,
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
                match &self.buffer_type {
                    BufferType::Directory => {},
                    _ => {
                        execute!(std::io::stderr(), cursor::SetCursorStyle::BlinkingBar).unwrap();
                        self.mode = mode;
                    }
                }
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
    pub fn insert_key_dir(&mut self, _key: KeyEvent) {
        // TODO: add functionality here
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

                let new_line: Line = Line { text: new_str, length: len };

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

                    line_above.text = format!("{}{}", line_above.text, &text);
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

    pub fn new_line_above(&mut self) {
        let current = self.ptr_y + self.cursor.current.1;
        // current = current.checked_sub(1).unwrap_or(0);
        let new_line = Line { text: "".to_string(), length: 0 };
        self.lines.lines.insert(current.into(), new_line);
        self.change_mode(Mode::Insert);
    }

    pub fn new_line_below(&mut self, size: (u16, u16)) {
        let mut current = self.ptr_y + self.cursor.current.1;
        current = current.checked_add(1).unwrap_or(u16::MAX);
        let new_line = Line { text: "".to_string(), length: 0 };
        self.lines.lines.insert(current.into(), new_line);
        self.move_down(size);
        self.change_mode(Mode::Insert);
    }

    // open file/dir under the cusor in dir menu and replace it in the current buffer
    // remember to reset cursor and anything else
    pub fn get_hover_file(&mut self) -> String {
        // idea:
        // get the file/dir name hovered over
        // convert to relative path
        // open
        // refresh buffer

        let line_index = usize::from(self.ptr_y + self.cursor.current.1);
        let current_line = &self.lines.lines[line_index];
        let file_name = current_line.text.clone();

        let res = &self.open(&file_name);

        if let Ok(_) = res {
            return file_name;
        }

        return "could not open file".to_string();
    }

    // TODO: grab current path and then join with new name 
    // impl as own function when making new buffer
    // ropes not impled here
    pub fn open(&mut self, name: &String) -> std::io::Result<()>{
        // convert name to relative path
        // check file vs dir
        // open and return
        let path = Path::new(name);

        if !path.exists() {
            return Err(std::io::Error::new(std::io::ErrorKind::Other, "Path didn't exist"))
        }

        if path.is_file() {
            if let Ok(file) = File::open(path) {
                self.buffer_type = BufferType::File;

                let reader = BufReader::new(file);
                let mut file_lines: Vec<Line> = vec![];

                for line in reader.lines() {
                    if let Ok(text) = line {
                        let length: u16 = text.len().try_into().unwrap();
                        file_lines.push(Line {
                            text,
                            length
                        });
                    }
                }

                // TODO: impl ropes
                self.lines = Lines { lines: file_lines, rope: Rope::new() };
                self.file = Some(path.to_owned());
            }
        } else if path.is_dir() {
            // WARNING: not implemented
            return Ok(());
        } else {
            panic!("no thank you");
        }

        self.refresh_buffer();

        Ok(())
    }

    pub fn refresh_buffer(&mut self) {
        self.cursor = Cursor::new();
        self.ptr_y = 0;
        self.ptr_x = 0;
        self.mode = Mode::Normal;
    }

    pub fn save(&self) -> String {
        if self.buffer_type == BufferType::File {
            let mut total_string = "".to_string();

            for line in self.lines.lines.iter() {
                total_string.push_str(&line.text);

                total_string.push('\n');
            }

            if let Some(file) = &self.file {
                let status = std::fs::write(file, total_string.as_bytes());

                match status {
                    Ok(_) => {
                        return format!("Wrote {} bytes", total_string.len())
                    },
                    Err(_) => String::from("writing to file didn't work"),
                }
            } else {
                return String::from("No file found")
            }
        } else {
            return String::from("Can't write to directory")
        }
    }
}
