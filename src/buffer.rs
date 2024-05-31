use std::path::{Path, PathBuf};
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::fs::{File, read_dir};
use crate::editor::{Cursor, Mode};
use crossterm::{cursor, execute};
use crate::word::{
    find_word_end_forward,
    find_word_start_forward,
    find_word_start_backward
};
use ropey::Rope;

/*

=== GETTING ROPES WORKING ===
   get UI working
   get movements working again
        * down 
        * up 
        * left 
        * rigt 
        * end of line 
   get text insertion and deletion working
        * Insertion 
            * new line (creation) 
        * deletion 
            * new (deletion) 
   get jump to line working, in editor.rs
        * general idea 
        * always displays off screen ~
   move_to functions
        * working state 
        * viewport calc 
            * refresh view 
   getting saving and action functions
        * save 
        * new line above 
        * new line below 
 7. open function
 8. opening directories
   remove any unused imports
=============================

*/


#[derive(PartialEq)]
pub enum BufferType {
    Empty,
    Directory,
    File
}

// fields will be added later
pub struct Lines{
    pub rope: Rope,
}

// TODO: might need to add a variable for pathing
// idea: replace file with path
pub struct Buffer {
    pub buffer_type: BufferType,
    pub lines: Lines,
    pub ptr_y: usize,
    pub ptr_x: usize,
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
                let rope = Rope::from_reader(
                    File::open(&path)?
                )?;
                lines = Lines { rope };
                file_path = Some(path.to_owned());
                btype = BufferType::File;
            } else if path.is_dir() {
                btype = BufferType::Directory;
                // TODO: impl ropes
                lines = Lines { rope: Rope::new() };

                // let self_dot = String::from(".");
                // let parent_dot = String::from("..");

                lines.rope.append(".\n".into());
                lines.rope.append("..\n".into());


                let reader = read_dir(path).unwrap();

                for path in reader {
                    let path = path.unwrap().file_name().into_string().unwrap();
                    // let len = path.len();
                    let mut path = String::from(path);
                    path.push_str("\n");
                    lines.rope.append(path.into());
                }

                lines.rope.remove(lines.rope.len_chars()-1..lines.rope.len_chars())
            } else {
                panic!("no thank you");
            }
        } else {
            btype = BufferType::Empty;
            // TODO: impl ropes
            lines = Lines { rope: Rope::new() }
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
                let slice = self.lines.rope.get_line(usize::from(self.cursor.current.1 + self.ptr_y));

                if let Some(slice) = slice {
                    let line_len = slice.len_chars();

                    if line_len == 0 {
                        self.cursor.current.0 = 0;
                    } else {
                        //let x = std::cmp::min(self.cursor.current.0, line_len - 1);
                        let x = std::cmp::min(self.cursor.current.0, line_len - 1);
                        self.cursor.current.0 = x;
                        self.cursor.possible.0 = x;
                    };

                    execute!(std::io::stderr(), cursor::SetCursorStyle::SteadyBlock).unwrap();
                    self.mode = mode;
                }
                
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
                    // inserting char into line
                    let line_idx = self.lines.rope.try_line_to_byte(self.ptr_y + self.cursor.current.1);

                    if let Ok(line_idx) = line_idx {
                        let res = self.lines.rope.try_insert_char(line_idx + self.ptr_x + self.cursor.current.0, value);
                        if let Ok(_) = res {
                            self.move_right();
                        }
                    }
                }
            },
            KeyCode::Enter => {
                // get current line index, append new line after

                // FIX: this breaks
                let line_idx = self.lines.rope.line_to_byte(self.ptr_y + self.cursor.current.1);
                self.lines.rope.insert_char(line_idx + self.cursor.current.0 + self.ptr_y, '\n');
                self.move_down(size);
                self.cursor.current.0 = 0;
                self.cursor.possible.0 = 0;
            },
            KeyCode::Backspace => {
                let line_idx = self.lines.rope.try_line_to_byte(self.ptr_y + self.cursor.current.1);

                if let Ok(line_idx) = line_idx {
                    let local_idx = self.cursor.current.0 + self.ptr_x;
                    let curr_idx = line_idx + local_idx;

                    if curr_idx <= 0 {
                        return;
                    }
                    
                    if local_idx == 0 {
                        // move up and end of line
                        self.move_up();
                        self.move_end_of_line()
                    } else {
                        self.move_left();
                    }
                    
                    let _ = self.lines.rope.try_remove(curr_idx-1..curr_idx);
                }
            },
            KeyCode::Tab => {
                // TODO: append tab
            },
            KeyCode::Esc => {
                self.change_mode(Mode::Normal);
            },
            _ => {}
        }
    }

    pub fn move_down(&mut self, size: (u16, u16)) {
        // next logical y
        let y = self.cursor.current.1.checked_add(1).unwrap_or(self.cursor.current.1);

        if y > size.1.checked_sub(1).unwrap_or(0).into() {
            if usize::from(size.1) + self.ptr_y < self.lines.rope.len_lines() - 1 {
                self.ptr_y += 1;
            }
        } else {
            // max lines in file
            let line_nums = self.lines.rope.len_lines() - 2;
            let cap = std::cmp::min(line_nums, usize::from(size.1 - 1) + self.ptr_y);
            let y = std::cmp::min(y, cap.try_into().unwrap());
            self.cursor.current.1 = y;
        }

        // setting cursor x dimension
        let line_len = self.lines.rope.get_line(self.cursor.current.1 + self.ptr_y).unwrap().len_chars() - 1;

        if line_len == 0 {
            self.cursor.current.0 = 0;
        } else {
            let x = std::cmp::max(self.cursor.current.0, self.cursor.possible.0);
            let x = std::cmp::min(x, line_len - 1);
            self.cursor.current.0 = x;
        }

        self.refresh_view(size);
    }

    pub fn move_up(&mut self) {
        if self.cursor.current.1 == 0 && self.ptr_y != 0 {
            self.ptr_y -= 1;
            return;
        }

        self.cursor.current.1 = self.cursor.current.1.checked_sub(1).unwrap_or(self.cursor.current.1);

        let line_len = self.lines.rope.get_line(self.cursor.current.1 + self.ptr_y).unwrap().len_chars();

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
        let line_len = self.lines.rope.get_line(self.cursor.current.1 + self.ptr_y).unwrap().len_chars() - 1;
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
        let slice = self.lines.rope.get_line(self.cursor.current.1 + self.ptr_y);

        if let Some(slice) = slice {
            let line_len = slice.len_chars() - 1;
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

    }

    pub fn move_begin_of_line(&mut self){
        self.cursor.current.0 = 0;
        self.cursor.possible.0 = 0;
    }

    // TODO: calc viewport if needed
    pub fn move_next_word(&mut self, size: (u16, u16)) {
        // FIX: calc viewport

        let str = self.lines.rope.to_string();
        let line_idx = self.lines.rope.try_line_to_byte(self.ptr_y + self.cursor.current.1);

        if let Ok(line_idx) = line_idx {
            let start_col = line_idx + self.ptr_x + self.cursor.current.0;
            let res = find_word_start_forward(&str, start_col);

            if let Some(idx) = res {
                // cursor movement
                let line_num = self.lines.rope.try_byte_to_line(idx);
                self.cursor.current.1 = line_num.unwrap_or(0);

                let line_idx = self.lines.rope.try_line_to_byte(self.ptr_y + self.cursor.current.1).unwrap_or(0);
                let line_pos = idx - line_idx;
                self.cursor.current.0 = line_pos;
            }
        }

        self.refresh_view(size);
    }

    // FIX: gross when executed, gets hung on an index
    pub fn move_end_word(&mut self, size: (u16, u16)) {
        let str = self.lines.rope.to_string();
        let line_idx = self.lines.rope.try_line_to_byte(self.ptr_y + self.cursor.current.1);

        if let Ok(line_idx) = line_idx {
            let start_col = line_idx + self.ptr_x + self.cursor.current.0;
            let res = find_word_end_forward(&str, start_col);

            if let Some(idx) = res {
                // cursor movement
                let line_num = self.lines.rope.try_byte_to_line(idx);
                self.cursor.current.1 = line_num.unwrap_or(0);

                let line_idx = self.lines.rope.try_line_to_byte(self.ptr_y + self.cursor.current.1).unwrap_or(0);
                let line_pos = idx - line_idx;
                self.cursor.current.0 = line_pos;
            }
        }

        self.refresh_view(size);
    }

    pub fn move_back_word(&mut self, size: (u16, u16)) {
        let str = self.lines.rope.to_string();
        let line_idx = self.lines.rope.try_line_to_byte(self.ptr_y + self.cursor.current.1);

        if let Ok(line_idx) = line_idx {
            let start_col = line_idx + self.ptr_x + self.cursor.current.0;
            let res = find_word_start_backward(&str, start_col);

            if let Some(idx) = res {
                // cursor movement
                let line_num = self.lines.rope.try_byte_to_line(idx);
                self.cursor.current.1 = line_num.unwrap_or(0);

                let line_idx = self.lines.rope.try_line_to_byte(self.ptr_y + self.cursor.current.1).unwrap_or(0);
                let line_pos = idx - line_idx;
                self.cursor.current.0 = line_pos;
            }
        }

        self.refresh_view(size);
    }

    pub fn new_line_above(&mut self, size: (u16, u16)) {
        self.cursor.current.0 = 0;
        self.cursor.possible.0 = 0;

        let line_idx = self.lines.rope.try_line_to_byte(self.ptr_y + self.cursor.current.1);

        if let Ok(idx) = line_idx {
            let _ = self.lines.rope.try_insert_char(idx, '\n');
            self.change_mode(Mode::Insert);
        }

        self.refresh_view(size);
    }

    pub fn new_line_below(&mut self, size: (u16, u16)) {
        /*
        let mut current = self.ptr_y + self.cursor.current.1;
        current = current.checked_add(1).unwrap_or(u16::MAX);
        let new_line = Line { text: "".to_string(), length: 0 };
        self.lines.lines.insert(current.into(), new_line);
        self.move_down(size);
        self.change_mode(Mode::Insert);
        */
        self.move_down(size);
        self.new_line_above(size);
        self.refresh_view(size);
    }

    // open file/dir under the cusor in dir menu and replace it in the current buffer
    // remember to reset cursor and anything else
    pub fn get_hover_file(&mut self) -> String {
        // idea:
        // get the file/dir name hovered over
        // convert to relative path
        // open
        // refresh buffer

        /*
        let line_index = usize::from(self.ptr_y + self.cursor.current.1);
        let current_line = &self.lines.lines[line_index];
        let file_name = current_line.text.clone();

        let res = &self.open(&file_name);

        if let Ok(_) = res {
            return file_name;
        }

        return "could not open file".to_string();
        */
        "".to_string()
    }

    // TODO: grab current path and then join with new name 
    // impl as own function when making new buffer
    // ropes not impled here
    // impl in new buffer
    pub fn open(&mut self, name: &String) -> std::io::Result<()>{
        /*
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

        */
        Ok(())
    }

    pub fn refresh_buffer(&mut self) {
        self.cursor = Cursor::new();
        self.ptr_y = 0;
        self.ptr_x = 0;
        self.mode = Mode::Normal;
    }

    pub fn refresh_view(&mut self, size: (u16, u16)) {
        // check if cursor on screen
        let current_line = self.ptr_y + self.cursor.current.1;

        // get maximum
        let max = self.ptr_y + size.1 as usize;
        // update accordingly

        if current_line > max {
            let adjustment = current_line - max;
            self.ptr_y += adjustment;
        }
    }

    // TODO: check file permissions
    pub fn save(&self) -> String {
        /*
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
        */
        if self.buffer_type == BufferType::File {
            // self.lines.rope.write_to(writer)
            // open file and wrap file writer 
            if let Some(file) = &self.file {
                let str = self.lines.rope.to_string();
                let status = std::fs::write(file, str.as_bytes());

                return match status {
                    Ok(_) => format!("Wrote {} bytes", str.len()),
                    Err(_) => String::from("Writing to file didn't work")
                };
            } else {
                return String::from("No file found");
            }
        }
        return String::from("Can't write to directory")
    }
}
