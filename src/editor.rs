use crate::command::{Command, CommandKey};
use crate::buffer::{Buffer, BufferType};
use crate::motion::MotionBuffer;
use crossterm::event::{KeyEvent, KeyCode, KeyModifiers};
use color_eyre::eyre::Result;
use std::io::Write;
use std::path::Path;
use std::usize;
use ratatui::style::Stylize;
use ratatui::prelude::{Style, Alignment};
use ratatui::widgets::{Paragraph, Borders, Block};
use std::net::TcpStream;

macro_rules! current_buf {
    ($e: expr) => {
        $e.buffers[$e.buf_ptr]
    };
}

pub enum Mode{
    Insert, 
    Command,
    Normal
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

pub struct Editor {
    pub buffers: Vec<Buffer>,
    pub buf_ptr: usize,
    pub command: Command,
    pub motion: MotionBuffer,
    pub should_quit: bool,
    pub size: (u16, u16),
    pub logger: Option<TcpStream>,
    pub message: Option<String>
}

impl Editor {
    pub fn new(path: &Path)-> Result<Editor> {
        let buf = Buffer::new(path)?;

        // port address for logger
        let port = match std::env::args().nth(2) {
            Some(value) => value,
            None => "".to_string()
        };

        if port != "" {
            //  TODO: 

            // old udp stuff
            //let socket = UdpSocket::bind("127.0.0.1:8080");

            let stream = TcpStream::connect(format!("127.0.0.1:{}", port));

            if let Ok(mut stream) = stream {
                // socket.connect(format!("127.0.0.1:{}", port)).unwrap();


                // TODO: this won't always work
                if stream.write(b"connection test").is_ok() {
                    return Ok(Editor {
                        buffers: vec![buf],
                        buf_ptr: 0,
                        command: Command::new(),
                        motion: MotionBuffer::new(),
                        should_quit: false,
                        size: (0, 0),
                        logger: Some(stream),
                        message: None,
                    });
                } 
            }
        } 

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

    pub fn change_mode(&mut self, mode: Mode) {
        current_buf!(self).change_mode(mode);

        // HACK: check if this worked

        match current_buf!(self).mode {
            Mode::Insert | Mode::Command => {
                self.set_message(None);
            }
            _ => (),
        }
    }

    // NOTE: display functions

    // FIX: this issue with the tcp logger
    pub fn mode_display(&mut self) -> (Paragraph, Option<Paragraph>) {
        match &current_buf!(self).mode {
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

                match &self.motion.action {
                    Some(value) => motion_str.push_str(value.clone().as_str()),
                    None => {}
                }
                
                let status = match &mut self.message {
                    Some(value) => value.to_owned(),
                    None => "-- Normal --".to_string(),
                };

                let status = Paragraph::new(format!("{}", status))
                        .block(Block::default()
                            .borders(Borders::TOP)
                            .border_style(Style::new().blue()));

                let motion = Paragraph::new(format!("{}", motion_str))
                    .block(Block::default()
                           .borders(Borders::TOP)
                           .border_style(Style::new().blue()))
                    .alignment(Alignment::Center);
                (status, Some(motion))
            },
            Mode::Command => {
                (Paragraph::new(format!(":{}", self.command.text)).block(Block::default().borders(Borders::TOP)), None)
            }
        }
    }


    /*
     * idea:
     * pass to function based off buffertype
     * empty will pretty much be same as file, will need modifications for saving
    */
    pub fn key_press(&mut self, key: KeyEvent){
        match current_buf!(self).buffer_type {
            BufferType::Directory => self.directory_key_press(key),
            BufferType::File => self.file_key_press(key),
            BufferType::Empty => {}, // FIX: add same to file, does nothing because saving not implemented
        }
    }

    // TODO: create function for handling commands so that they aren't handled in these functions
    // add permission checks to buffers when saving
    fn directory_key_press(&mut self, key: KeyEvent){
        match current_buf!(self).mode {
            Mode::Command => self.command_line_key(key),
            Mode::Normal => {

                // TODO: change this impl to work with opening buffers and such
                // will have to create functions to handle operations
                match key.code {
                    KeyCode::Enter => {
                        // open file/directory
                        let file_name = current_buf!(self).get_hover_file();
                        self.send(format!("Opening {file_name}"));
                    },
                    KeyCode::Char(value) => {
                        if value == 'c' && key.modifiers == KeyModifiers::CONTROL {
                            self.motion.clear();
                        } else {
                            let res = self.motion.push(value);

                            match res {
                                Some(_) => {
                                    // maybe will use, maybe not, whos to say
                                    //let _ = self.direc_parse();
                                    let _ = self.parse();
                                    self.motion.clear();
                                },
                                None => {}
                            }
                        }
                    },
                    _ => {}
                }
            }
            _ => {},
        }
    }

    fn file_key_press(&mut self, key: KeyEvent){
        match current_buf!(self).mode {
            Mode::Insert => self.insert_key(key),
            Mode::Command => self.command_line_key(key),
            Mode::Normal => {
                match key.code {
                    KeyCode::Char(value) => {
                        if value == 's' && key.modifiers == KeyModifiers::CONTROL {
                            let update = self.save();
                            self.set_message(Some(update.clone()));
                        } else if value == 'c' && key.modifiers == KeyModifiers::CONTROL {
                            self.motion.clear();
                        } else {
                            let res = self.motion.push(value);

                            match res {
                                Some(_) => {
                                    let _ = self.parse();
                                    self.motion.clear();
                                },
                                None => {}
                            }
                        }
                    },
                    _ => {}
                }
            }
        }
    }

    pub fn command_line_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Char(value) => {
                if value == 'c' && key.modifiers == KeyModifiers::CONTROL {
                    self.change_mode(Mode::Normal);
                } else {
                    self.command.text.push(value);
                }
            },
            KeyCode::Esc => {
                self.change_mode(Mode::Normal);
            }
            KeyCode::Enter => {
                let command = self.command.confirm();
                self.handle_command(command);
                self.change_mode(Mode::Normal);
            },
            KeyCode::Backspace => {
                if self.command.text.len() > 0 {
                    // TODO: add movable cursor
                    self.command.text.pop();
                } else {
                    self.change_mode(Mode::Normal);
                }
            },
            _ => {}
        }
    }
    
    // NOTE: not specifically for inserting a key, but key handling in insert mode
    //
    // TODO: figure out what this is for
    pub fn insert_key(&mut self, key: KeyEvent) {
        match &current_buf!(self).buffer_type {
            BufferType::Directory => {
                current_buf!(self).insert_key_dir(key);
            },
            _ => {
                current_buf!(self).insert_key_file(key, self.size);
            }
        }
    }

    // NOTE: word movements

    // TODO: needs to recalculate the viewpoint, won't be too bad
    pub fn go_to_line(&mut self, index: usize) {
        /* FIX: impl this for rope
        if index == 0 {
            return
        }

        let index = index - 1;
        if index < current_buf!(self).lines.lines.len() {
            current_buf!(self).cursor.current.0 = index;
            current_buf!(self).cursor.current.1 = index;
        } 
        */
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

        // TODO: figure out how have these commands run
        let action = match &self.motion.action {
            Some(value) => value.clone(),
            None => "".to_string(),
        };

        let action_args = match &self.motion.action_arg {
            Some(value) => value.clone(),
            None => "".to_string(),
        };

        for _ in 0..count {
            // perform action then move cursor
            self.motion_func(&motion);

            match &current_buf!(self).mode {
                Mode::Normal => {},
                _ => break,
            }
        }


        for _ in 0..count {
            // perform action then move cursor
            self.action_func(&action, &action_args);

            match &current_buf!(self).mode {
                Mode::Normal => {},
                _ => break,
            }
        }

        Ok(0)
    }

    // FIX: might not be great
    pub fn direc_parse(&mut self) -> Result<u32, &str> {
        Ok(0)
    }

    pub fn handle_command(&mut self, command: Option<CommandKey>){
        match command {
            Some(command) => {
                match command {
                    CommandKey::Save => {
                        let update = self.save();
                        self.set_message(Some(update))
                    },
                    CommandKey::Quit => self.close_buffer(),
                    CommandKey::Line(number) => {
                        self.go_to_line(number);
                    },
                    CommandKey::SaveAndQuit => {
                        self.save();
                        self.should_quit = true;
                    },
                    CommandKey::History => todo!(),
                    CommandKey::Logger => {
                        // TODO: finish this up
                        let output = match &self.logger {
                            Some(socket) => {
                                let addr = socket.local_addr().unwrap().to_string();
                                format!("Binded to {}", addr)
                            },
                            None => "Not Connected".to_string()
                        };
                        self.set_message(Some(output))
                    },
                    CommandKey::Send(message) => {
                        // TODO: make function for sending
                        self.send(message);
                    },
                    CommandKey::NextBuf => {
                        self.next_buf();
                        self.send(String::from(format!("buf: {}", self.buf_ptr)));
                    },
                    CommandKey::PrevBuf => {
                        self.prev_buf();
                        self.send(String::from(format!("buf: {}", self.buf_ptr)));
                    },
                    CommandKey::NewBuf => {
                        self.send(String::from("New buffer"));
                        self.new_buffer(std::path::Path::new("."));
                    },
                    CommandKey::BufCount => {
                        // sent message to count of opened buffers
                        let message = String::from(format!("{} open buffers", self.buffers.len()));
                        self.set_message(Some(message))
                    }
                }
            },
            None => {}
        }
    }

    pub fn motion_func(&mut self, key: &String) {
        match key.as_str() {
            ":" => current_buf!(self).change_mode(Mode::Command),
            "j" => current_buf!(self).move_down(self.size),
            "k" => current_buf!(self).move_up(),
            "h" => current_buf!(self).move_left(),
            "l" => current_buf!(self).move_right(),
            "i" => current_buf!(self).change_mode(Mode::Insert),
            "a" => {
                current_buf!(self).change_mode(Mode::Insert);
                current_buf!(self).move_right();
            },
            "O" => {
                current_buf!(self).new_line_above();
            },
            "o" => {
                current_buf!(self).new_line_below(self.size);
            },
            "w" => current_buf!(self).move_next_word(),
            "b" => current_buf!(self).move_back_word(),
            "e" => current_buf!(self).move_end_word(),
            "0" => current_buf!(self).move_begin_of_line(),
            "$" => current_buf!(self).move_end_of_line(),
            "I" => {
                current_buf!(self).change_mode(Mode::Insert);
                current_buf!(self).move_begin_of_line();
            },
            "A" => {
                current_buf!(self).change_mode(Mode::Insert);
                current_buf!(self).move_end_of_line();
            },
            _ => {}
        }
    }

    // TODO: this will be used for actions, will need action_args
    pub fn action_func(&mut self, key: &String, _args: &String){
        match key.as_str() {
            "d" => {}
            "s" => {}
            "f" => {}
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
                // log out error to tcp logger
                self.send("Can't make buffer".to_string());
            },
        }
    }

    // TODO: figure how this should be handled if this is only buffer
    // reset buf_ptr, ++/--
    pub fn close_buffer(&mut self){
        if self.buffers.len() == 1 {
            self.should_quit = true;
            return;
        }

        self.buffers.remove(self.buf_ptr);
        
        if self.buf_ptr > 0 {
            self.buf_ptr -= 1;
        }

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

    pub fn set_message(&mut self, new_mes: Option<String>) {
        match new_mes {
            Some(message) => {
                self.message = Some(message.clone());
                self.send(format!("Set Message {}", message.clone()))
            },
            None => self.message = None
        }
    }

    // NOTE: saving functions

    // TODO: make this safer by reading permissions
    pub fn save(&mut self) -> String {
        // FIX: too much extra memory
        return current_buf!(self).save();
    }

    // NOTE: functions for logging
    //
    // FIX: THIS MESS
    pub fn send(&mut self, message: String){
        let _ = {
            match &mut self.logger {
                Some(stream) => {
                    let _ = stream.write(message.as_bytes());
                },
                None => {}
            }
        };
    }
}
