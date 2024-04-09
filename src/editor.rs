use color_eyre::eyre::Result;
use std::path::Path;
use std::usize;
use crossterm::event::KeyEvent;
use ratatui::prelude::Alignment;
use ratatui::widgets::{Paragraph, Borders, Block};
use crate::command::Command;
use crate::motion::MotionBuffer;
use std::net::UdpSocket;
use crate::buffer::{Buffer, BufferType};

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

pub struct Editor {
    pub buffers: Vec<Buffer>,
    pub buf_ptr: usize,
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

        if port == "" || port == "8080"{
            return Ok(Editor {
                buffers: vec![buf],
                buf_ptr: 0,
                command: Command::new(),
                motion: MotionBuffer::new(),
                should_quit: false,
                size: (0, 0),
                logger: None,
                message: None
            });
        } else {
            //  TODO: connect to udp socket here and save socket to logger
            let socket = UdpSocket::bind("127.0.0.1:8080").unwrap();
            socket.connect(format!("127.0.0.1:{}", port)).unwrap();

            // TODO: this won't always work
            if socket.send(b"connection test").is_ok() {
                return Ok(Editor {
                    buffers: vec![buf],
                    buf_ptr: 0,
                    command: Command::new(),
                    motion: MotionBuffer::new(),
                    should_quit: false,
                    size: (0, 0),
                    logger: Some(socket),
                    message: None
                });
            } else {
                return Ok(Editor {
                    buffers: vec![buf],
                    buf_ptr: 0,
                    command: Command::new(),
                    motion: MotionBuffer::new(),
                    should_quit: false,
                    size: (0, 0),
                    logger: None,
                    message: None
                });
            }
        }
    }

    pub fn change_mode(&mut self, mode: Mode) {
        self.message = None;
        self.buffers[self.buf_ptr].change_mode(mode);
    }

    // NOTE: display functions

    pub fn mode_display(&self) -> (Paragraph, Option<Paragraph>) {
        match &self.buffers[self.buf_ptr].mode {
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
        match &self.buffers[self.buf_ptr].b_type {
            BufferType::Directory => {
                self.buffers[self.buf_ptr].insert_key_dir(key);
            },
            _ => {
                self.buffers[self.buf_ptr].insert_key_file(key, self.size);
            }
        }
    }

    // NOTE: word movements

    // TODO: needs to recalculate the viewpoint
    pub fn go_to_line(&mut self, index: usize) {
        let index = index - 1;
        if index < self.buffers[self.buf_ptr].lines.lines.len() {
            self.buffers[self.buf_ptr].cursor.current.0 = index as u16;
            self.buffers[self.buf_ptr].cursor.current.1 = index as u16;
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

            match &self.buffers[self.buf_ptr].mode {
                Mode::Normal => {},
                _ => break,
            }
        }

        Ok(0)
    }

    pub fn motion_func(&mut self, key: &String) {
        match key.as_str() {
            ":" => self.buffers[self.buf_ptr].change_mode(Mode::Command),
            "j" => self.buffers[self.buf_ptr].move_down(self.size),
            "k" => self.buffers[self.buf_ptr].move_up(),
            "h" => self.buffers[self.buf_ptr].move_left(),
            "l" => self.buffers[self.buf_ptr].move_right(),
            "i" => self.buffers[self.buf_ptr].change_mode(Mode::Insert),
            "a" => {
                self.buffers[self.buf_ptr].change_mode(Mode::Insert);
                self.buffers[self.buf_ptr].move_right();
            },
            "O" => {
                self.buffers[self.buf_ptr].new_line_above();
            },
            "o" => {
                self.buffers[self.buf_ptr].new_line_below(self.size);
            },
            "w" => self.buffers[self.buf_ptr].move_next_word(),
            "b" => self.buffers[self.buf_ptr].move_back_word(),
            "e" => self.buffers[self.buf_ptr].move_end_word(),
            "0" => self.buffers[self.buf_ptr].move_begin_of_line(),
            "$" => self.buffers[self.buf_ptr].move_end_of_line(),
            "I" => {
                self.buffers[self.buf_ptr].change_mode(Mode::Insert);
                self.buffers[self.buf_ptr].move_begin_of_line();
            },
            "A" => {
                self.buffers[self.buf_ptr].change_mode(Mode::Insert);
                self.buffers[self.buf_ptr].move_end_of_line();
            },
            _ => {}
        }
    }

    // TODO: add function for modifying what the buffer contains

    pub fn new_buffer(&mut self, path: &Path){
        // WARN: check for possible errors that can return
        let buf = Buffer::new(path);

        match buf {
            Ok(buf) => {
                self.buffers.push(buf);
                self.next_buf();
            },
            Err(_) => {
                // log out error to udp socket
                match &self.logger {
                    Some(socket) => {
                        let _ = socket.send(b"Can't make buffer");
                    },
                    None => {}
                }
            },
        }
    }

    #[warn(unused)]
    pub fn close_buffer(&mut self){
        self.buffers.remove(self.buf_ptr);

        // TODO: figure how this should be handled if this is only buffer
        // reset buf_ptr, ++/--
    }

    pub fn next_buf(&mut self) {
        // make cycling buffer wheel
        let max = self.buffers.len();

        let current = self.buf_ptr.checked_add(1).unwrap_or(0) % max;

        self.buf_ptr = current;
    }

    pub fn prev_buf(&mut self) {
        let next = self.buf_ptr.checked_sub(1);

        match next {
            Some(value) => {
                self.buf_ptr = value;
            },
            None => {
                self.buf_ptr = self.buffers.len().checked_sub(1).unwrap();
            }
        }
    }

    // NOTE: saving functions

    pub fn save(&mut self) {
         // NOTE: too much extra memory
        self.buffers[self.buf_ptr].save();
    }

    // NOTE: functions for logging
    #[warn(unused)]
    pub fn send(&self, message: String){
        let _output = {
            match &self.logger {
                Some(socket) => {
                    let _ = socket.send(message.as_bytes());
                },
                None => {}
            }
        };
    }
}
