use color_eyre::eyre::Result;
use std::path::Path;
use std::usize;
use crossterm::event::KeyEvent;
use ratatui::prelude::Alignment;
use ratatui::widgets::{Paragraph, Borders, Block};
use crate::command::Command;
use crate::motion::MotionBuffer;
use std::net::UdpSocket;
use crate::buffer::Buffer;


pub enum Mode{
    Insert, 
    Command,
    Normal
}

pub struct Cursor{
    pub current: (u16, u16),
    pub possible: (u16, u16)
}

impl Cursor{
    pub fn new() -> Cursor{
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

// TODO: fix what happens on resize
pub struct Editor {
    pub buffer: Buffer,
    pub command: Command,
    pub motion: MotionBuffer,
    pub should_quit: bool,
    pub size: (u16, u16),
    pub logger: Option<UdpSocket>,
    pub message: Option<String>
}

impl Editor {
    pub fn new(path: &Path)-> Result<Editor> {
        let buf = Buffer::new(path)?;
        // port address for logger
        let port = match std::env::args().nth(2) {
            Some(value) => {
                value
            },
            None => "".to_string()
        };

        if port == "" || port == "8000"{
            return Ok(Editor {
                buffer: buf,
                command: Command::new(),
                motion: MotionBuffer::new(),
                should_quit: false,
                size: (0, 0),
                logger: None,
                message: None
            });
        } else {
            //  TODO: connect to udp socket here and save socket to logger
            let socket = UdpSocket::bind("127.0.0.1:8000").unwrap();
            socket.connect(format!("127.0.0.1:{}", port)).unwrap();

            return Ok(Editor {
                buffer: buf,
                command: Command::new(),
                motion: MotionBuffer::new(),
                should_quit: false,
                size: (0, 0),
                logger: Some(socket),
                message: None
            });
        }
    }

    pub fn change_mode(&mut self, mode: Mode) {
        self.buffer.change_mode(mode);
    }

    // NOTE: display functions

    pub fn mode_display(&self) -> (Paragraph, Option<Paragraph>) {
        match &self.buffer.mode {
            Mode::Insert => {
                (Paragraph::new("-- Insert --").block(Block::default().borders(Borders::TOP)), None)
            },
            Mode::Normal => {
                // TODO: temp idea for displaying motions
                let mut motion_str = "".to_string();

                match &self.motion.number {
                    Some(value) => motion_str.push_str(value.clone().as_str()),
                    None => {}
                }

                match &self.motion.command {
                    Some(value) => motion_str.push_str(value.clone().as_str()),
                    None => {}
                }
                
                let status = match &self.message {
                    Some(value) => {
                        value.to_owned()
                    },
                    None => "-- Normal --".to_string()
                };

                let status = Paragraph::new(format!("{}", status) ).block(Block::default().borders(Borders::TOP));
                let motion = Paragraph::new(format!("{}", motion_str)).block(Block::default().borders(Borders::TOP)).alignment(Alignment::Center);
                (status, Some(motion))
            },
            Mode::Command => {
                (Paragraph::new(format!(":{}", self.command.text)).block(Block::default().borders(Borders::TOP)), None)
            }
        }
    }

    
    // NOTE: not specifically for inserting a key, but key handling in insert mode
    pub fn insert_key(&mut self, key: KeyEvent) {
        // NOTE: temp
        self.buffer.insert_key_file(key, self.size);
    }

    // NOTE: word movements
    pub fn go_to_line(&mut self, index: usize) {
        let index = index - 1;
        if index < self.buffer.lines.lines.len() {
            self.buffer.cursor.current.0 = index as u16;
            self.buffer.cursor.current.1 = index as u16;
        } 
    }


    // NOTE: motion parsing function
        // might have to be async for timming
            //  could possile use channels for this
        //  might need an action function
        //  will probably remove returned result
    pub fn parse(&mut self) -> Result<u32, &str> {
        let count = match &self.motion.number{
            Some(value) => value.parse::<u32>().unwrap_or(0),
            None => 1,
        };

        let motion = match &self.motion.motion {
            Some(value) => value.clone(),
            None => "".to_string(),
        };

        let _command = match &self.motion.command {
            Some(value) => value.clone(),
            None => "".to_string(),
        };

        for _ in 0..count {
            // perform action then move cursor
            self.motion_func(&motion);

            match &self.buffer.mode {
                Mode::Normal => {},
                _ => break,
            }
        }

        Ok(0)
    }

    pub fn motion_func(&mut self, key: &String) {
        match key.as_str() {
            ":" => self.buffer.change_mode(Mode::Command),
            "j" => self.buffer.move_down(self.size),
            "k" => self.buffer.move_up(),
            "h" => self.buffer.move_left(),
            "l" => self.buffer.move_right(),
            "i" => self.buffer.change_mode(Mode::Insert),
            "a" => {
                self.buffer.change_mode(Mode::Insert);
                self.buffer.move_right();
            },
            "w" => self.buffer.move_next_word(),
            "b" => self.buffer.move_back_word(),
            "e" => self.buffer.move_end_word(),
            "0" => self.buffer.move_begin_of_line(),
            "$" => self.buffer.move_end_of_line(),
            "I" => {
                self.buffer.change_mode(Mode::Insert);
                self.buffer.move_begin_of_line();
            },
            "A" => {
                self.buffer.change_mode(Mode::Insert);
                self.buffer.move_end_of_line();
            },
            _ => {}
        }
    }

    // NOTE: saving functions


    // BUG: most likely definetly won't work
    pub fn save(&mut self) {
         // NOTE: too much extra memory

        let mut total_string = "".to_string();

        for line in self.buffer.lines.lines.iter() {
            total_string.push_str(&line.text);

            total_string.push('\n');
        }

        if let Some(file) = &self.buffer.file {
            let status = std::fs::write(file, total_string.as_bytes());

            match status {
                Ok(_) => {},
                Err(_) => panic!("writing to file didn't work"),
            }
        } else {
            return;
        }
    }
}
