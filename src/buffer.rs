use std::path::{Path, PathBuf};
use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::env;
use std::fs::{
    File,
    read_dir,
};
use ratatui::{
    style::{Color, Style},
    widgets::{Block, Padding, Paragraph},
    // prelude::{Span, Line},
};
use tree_sitter_rust;
use tree_sitter_highlight::{
    // Highlighter,
    // HighlightEvent,
    HighlightConfiguration,
};
use crossterm::{cursor, execute};
use crate::word::{
    find_word_end_forward,
    find_word_start_forward,
    find_word_start_backward
};
use ropey::Rope;

/*

=============================
 1. syntax highlighting with tree sitter
 2. opening symlinks
=============================

*/

#[warn(dead_code)]
enum FileType {
    Rust
}

#[derive(Clone, Copy)]
pub enum Mode{
    Insert, 
    Command,
    Normal,
}

pub struct Cursor{
    pub current: (usize, usize),
    pub possible: (usize, usize)
}

impl Cursor{
    pub fn new() -> Cursor{
        Cursor { current: (0,0), possible: (0,0) }
    }
}


#[derive(PartialEq)]
pub enum BufferType {
    Empty,
    Directory,
    File
}

pub struct Lines{
    pub rope: Rope,
}

// TODO: might need to add a variable for pathing
// idea: replace file with path
pub struct Buffer {
    pub buffer_type: BufferType,
    pub lines: Lines,
    pub size: (u16, u16),
    pub ptr_y: usize,
    pub ptr_x: usize,
    pub cursor: Cursor,
    pub file: Option<PathBuf>,
    pub parent_dir: Option<PathBuf>,
    pub mode: Mode,
}


impl Buffer {
    // TODO: rework to incorporate open
    pub fn new(path: &String, window_size: (u16, u16)) -> Result<Buffer> {
        let parent_dir = env::current_dir()?;

        let mut buffer = Buffer {
            buffer_type: BufferType::Empty,
            lines: Lines {
                rope: Rope::new(),
            },
            size: window_size,
            ptr_y: 0,
            ptr_x: 0,
            cursor: Cursor::new(),
            file: None,
            parent_dir: Some(parent_dir),
            mode: Mode::Normal,
        };

        buffer.open(path)?;

        Ok(buffer)
    }

    // NOTE: mode change functions

    pub fn change_mode(&mut self, mode: Mode) {
        match mode {
            Mode::Insert => {
                match &self.buffer_type {
                    BufferType::Directory => {},
                    _ => {
                        execute!(std::io::stderr(), cursor::SetCursorStyle::SteadyBar).unwrap();
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
    // and improve error handling
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
                let line_pos = idx.checked_sub(line_idx).unwrap_or(line_idx);
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
        self.move_down(size);
        self.new_line_above(size);
        self.refresh_view(size);
    }

    pub fn get_hover_file(&mut self) -> String {
        let line_idx = self.ptr_y + self.cursor.current.1;
        let current_line = self.lines.rope.get_line(line_idx);

        if let Some(line) = current_line {
            let mut result = "".to_string();
            for c in line.chars() {
                if c == '\n' {
                    break;
                }
                result.push(c);
            }

            self.refresh_buffer();
            return result;
        }

        return "could not open file".to_string();
    }

    // TODO: impl in new buffer
    pub fn open(&mut self, name: &String) -> std::io::Result<()>{
        let path = Path::new(name);
        if let Some(parent_dir) = &self.parent_dir {
            let path = parent_dir.join(path);

            // TODO: impl for empty
            if !path.exists() {
                self.buffer_type = BufferType::Empty;
                self.lines.rope = Rope::new();
                self.file = None;
                return Ok(())
            }

            if path.is_file() {
                let rope = Rope::from_reader(
                    File::open(&path)?
                    )?;

                self.lines.rope = rope;
                self.file = Some(path.to_owned());
                self.buffer_type = BufferType::File;
            } else if path.is_dir() {
                // children to lines in rope
                let mut rope = Rope::new();

                rope.append(".\n".into());
                rope.append("..\n".into());

                let reader = read_dir(path.clone()).unwrap();

                for path in reader {
                    let path = path.unwrap().file_name().into_string().unwrap();
                    let mut path = String::from(path);
                    path.push_str("\n");
                    rope.append(path.into());
                }

                self.lines.rope = rope;
                self.file = None;
                self.parent_dir = Some(path);
                self.buffer_type = BufferType::Directory;
            } else if path.is_symlink() {
                todo!();
            } else {
                panic!("no thank you");
            }

            self.refresh_buffer();
            Ok(())
        } else {
            panic!("how did you manage to break this");
        }
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
        let max = self.ptr_y + size.1 as usize;

        if current_line > max {
            let adjustment = current_line - max;
            self.ptr_y += adjustment;
        }
    }

    // TODO: check file permissions
    pub fn save(&self) -> String {
        if self.buffer_type == BufferType::File {
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

    #[warn(dead_code)]
    pub fn resize(&mut self, new_size: (u16, u16)) {
        self.size = new_size;
    }

    // if returns some then I can use tree sitter
    // will be needed for rendering
    /*
    fn get_valid_file_type(&self) -> Option<FileType> {
        if let Some(path_buf) = &self.file {
            let extension = path_buf.as_path().extension();
            if let Some(extension) = extension {
                if "rs" == extension {
                    return Some(FileType::Rust);
                }
            }
        }

        None
    }

    fn create_rust_ts_config(&self) -> HighlightConfiguration {
        let highlight_names = [
            "keyword",
        ];

        let mut config = HighlightConfiguration::new(
            tree_sitter_rust::language(),
            "rust",
            tree_sitter_rust::HIGHLIGHTS_QUERY,
            tree_sitter_rust::INJECTIONS_QUERY,
            tree_sitter_rust::TAGS_QUERY,
        ).unwrap();

        config.configure(&highlight_names);

        config
    }
    */
}


// TODO: rename functions in this
impl <'a>Buffer {
    // line nums, text field
    pub fn ui(&self) -> (Paragraph<'a>, Paragraph<'a>) {
        /*
        let file_type = self.get_valid_file_type();

        if let Some(t) = file_type {
            match t {
                // create config and then pass to 'complicated' renderer
                FileType::Rust => {
                    // HACK: might cause some slow down
                    let _config = self.create_rust_ts_config();
                    return self.complicated_paragraphs(_config);
                }
            }
        }
        */

        let (line_string, text_string) = self.basic_text();

        let line_par = Paragraph::new(line_string)
                        .alignment(ratatui::layout::Alignment::Right)
                        .style(Style::default().fg(Color::DarkGray));

        let text_par = Paragraph::new(text_string)
                        .block(Block::default()
                               .padding(Padding::new(1, 0, 0, 0)));

        (line_par, text_par) 
    }

    /*
    fn complicated_paragraphs(&self, config: HighlightConfiguration) -> (Paragraph<'a>, Paragraph<'a>) {
        let mut hl = Highlighter::new();
        let text = self.lines.rope.to_string();

        let highlights = hl.highlight(
            &config,
            text.as_bytes(),
            None,
            |_| None
        ).unwrap();

        let mut active_style: usize = 0;
        let mut lines = vec![];

        let starting_line = self.lines.rope.line_to_byte(self.ptr_y);
        // let ending_line = self.lines.rope.try_line_to_byte(self.ptr_y + self.size.1 as usize).unwrap_or(self.lines.rope.len_chars());

        for highlight_event in highlights {
            match highlight_event.unwrap() {
                // indicates what highlighter to use 
                HighlightEvent::HighlightStart(s) => active_style = s.0,

                // indicates range
                HighlightEvent::Source { start, end } => {
                    // TODO: do better checking here for eliminating loops
                    if starting_line < start {
                        continue;
                    }

                    // do line creation and push
                    let mut t = String::new();
                    t.clone_from(&text[start..end].to_string());

                    let s: Span;
                    if active_style != 0 {
                        s = Span::styled(t, Style::default().fg(Color::Cyan));
                    } else {
                        s = Span::styled(t, Style::default());
                    }
                    lines.push(s);
                },

                HighlightEvent::HighlightEnd => {},
            }
        }

        // TODO: number lines with color
        (Paragraph::new("no"),
            Paragraph::new(lines)
             .block(
                 Block::default()
                 .padding(Padding::new(1, 0, 0, 0))
                 )
         )
    }
    */

    fn basic_text(&self) -> (String, String) {
        let mut line_nums = "".to_string();
        let mut text_string = "".to_string();

        for (i, line) in self.lines.rope.lines().skip(self.ptr_y).enumerate() {
            if i > self.ptr_y + usize::from(self.size.1) ||
                i == self.lines.rope.len_lines() - 1 {
                    break;
            }

            let mut i_str: String;
            let current_line = usize::from(self.cursor.current.1);

            if current_line != i {
                if current_line > i {
                    i_str = (current_line - i).to_string();
                } else{
                    i_str = (i - current_line).to_string();
                }

            } else {
                i_str = (self.ptr_y + self.cursor.current.1 + 1).to_string();
                if i_str.len() <= 2 {
                    i_str.push(' ');
                }
            }

            i_str.push_str("\n\r");

            for char in i_str.chars() {
                line_nums.push(char);
            }

            text_string.push_str(&line.to_string());
        }

        (line_nums, text_string)
    }
}
