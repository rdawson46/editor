use editor_core::{
    buffer::{Buffer, BufferType, Mode},
    command::{Command, CommandKey},
    X_OFFSET,
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent};
use color_eyre::eyre::Result;
use std::{
    io::Write, net::TcpStream, usize
};
use ratatui::{
    prelude::Style,
    style::Stylize,
    widgets::{
        Block,
        Borders,
        Paragraph,
    },
    Frame,
};
use tokio::{
    sync::mpsc,
    sync::mpsc::{
        UnboundedReceiver,
        UnboundedSender,
    }
};

/*
macro_rules! current_win {
    ($e: expr) => {
        $e.windows[$e.win_ptr]
    };
}
*/

pub struct Editor {
    pub buffers: Vec<Buffer>,
    // pub windows: Vec<Window>,
    // pub win_ptr: usize,
    pub buf_ptr: usize,
    pub command: Command,
    pub should_quit: bool,
    pub size: (u16, u16),
    pub logger: Option<TcpStream>,
    pub message: Option<String>,

    pub motion_sender: UnboundedSender<char>,
    pub clear_sender: UnboundedSender<bool>,
    pub motion_listener: UnboundedReceiver<Vec<String>>,
}

impl Editor {
    pub fn new(motion_sender: mpsc::UnboundedSender<char>, clear_sender: mpsc::UnboundedSender<bool>, motion_buffer_listener: mpsc::UnboundedReceiver<Vec<String>>) -> Result<Editor> {
        // port address for logger
        let port = match std::env::args().nth(2) {
            Some(value) => value,
            None => "".to_string()
        };

        if port != "" {
            let stream = TcpStream::connect(format!("127.0.0.1:{}", port));

            if let Ok(mut stream) = stream {
                if stream.write(b"connection test").is_ok() {
                    return Ok(Editor {
                        buffers: vec![],
                        buf_ptr: 0,
                        command: Command::new(),
                        should_quit: false,
                        size: (0, 0),
                        logger: Some(stream),
                        message: None,

                        motion_listener: motion_buffer_listener,
                        motion_sender,
                        clear_sender,
                    });
                } 
            }
        } 

        return Ok(Editor {
            buffers: vec![],
            buf_ptr: 0,
            command: Command::new(),
            should_quit: false,
            size: (0, 0),
            logger: None,
            message: None,

            motion_listener: motion_buffer_listener,
            motion_sender,
            clear_sender,
        });
    }

    fn current_buffer(&self) -> Option<&Buffer> {
        self.buffers.get(self.buf_ptr)
    }

    fn current_buffer_mut(&mut self) -> Option<&mut Buffer> {
        self.buffers.get_mut(self.buf_ptr)
    }

    pub fn change_mode(&mut self, mode: Mode) {
        if let Some(buffer) = self.current_buffer_mut() {
            buffer.change_mode(mode);
        }

        match mode {
            Mode::Insert | Mode::Command => {
                self.set_message(None);
                self.command.clear();
            }
            _ => (),
        }
    }

    // NOTE: display functions
    //
    // TODO: find way to get motion string
    pub fn mode_display(&mut self) -> Paragraph {
        if let Some(buffer) = self.current_buffer() {
            match &buffer.mode {
                Mode::Insert => {
                    Paragraph::new("-- Insert --").block(Block::default().borders(Borders::TOP))
                }
                Mode::Normal => {
                    let status = match &mut self.message {
                        Some(value) => value.to_owned(),
                        None => "-- Normal --".to_string(),
                    };

                    let status = Paragraph::new(format!("{}", status))
                        .block(Block::default().borders(Borders::TOP).border_style(Style::new().blue()));

                    status
                }
                Mode::Command => {
                    Paragraph::new(format!(":{}", self.command.text))
                        .block(Block::default().borders(Borders::TOP))
                }
                Mode::Visual { .. } => todo!("impl visual mode for ui"),
            }
        } else {
            Paragraph::new("")
        }
    }

    // NOTE: event functions
    /*
     * idea:
     * pass to function based off buffertype
     * empty will pretty much be same as file, will need modifications for saving
     */
    pub fn key_press(&mut self, key: KeyEvent) {
        let buffer_type = self.current_buffer().map(|b| b.buffer_type);
        if let Some(buffer_type) = buffer_type {
            match buffer_type {
                BufferType::Directory => self.directory_key_press(key),
                BufferType::File => self.file_key_press(key),
                BufferType::Empty => {} // FIX: add same to file, does nothing because saving not implemented
            }
        }
    }

    // TODO: create function for handling commands so that they aren't handled in these functions
    // add permission checks to buffers when saving
    fn directory_key_press(&mut self, key: KeyEvent) {
        let mode = match self.current_buffer() {
            Some(b) => b.mode,
            None => return,
        };

        match mode {
            Mode::Command => self.command_line_key(key),
            Mode::Normal => {
                // will have to create functions to handle operations
                match key.code {
                    KeyCode::Enter => {
                        // open file/directory
                        let file_name = self.current_buffer_mut().map(|b| b.get_hover_file());

                        if let Some(file_name) = file_name {
                            self.send(format!("Opening {file_name}"));
                            if let Some(b) = self.current_buffer_mut() {
                                let _ = b.open(&file_name);
                            }
                        }
                    }
                    KeyCode::Char(value) => {
                        if value == 'c' && key.modifiers == KeyModifiers::CONTROL {
                            let _ = self.clear_sender.send(true);
                        } else {
                            let _ = self.motion_sender.send(value);
                        }
                    }
                    _ => {}
                }
            }
            _ => {}
        }
    }

    fn file_key_press(&mut self, key: KeyEvent) {
        let mode = match self.current_buffer() {
            Some(b) => b.mode,
            None => return,
        };
        match mode {
            Mode::Insert => self.insert_key(key),
            Mode::Command => self.command_line_key(key),
            Mode::Normal => {
                // this will create issue moving forward
                match key.code {
                    KeyCode::Char(value) => {
                        if value == 's' && key.modifiers == KeyModifiers::CONTROL {
                            let update = self.save();
                            self.set_message(Some(update.clone()));
                        } else if value == 'c' && key.modifiers == KeyModifiers::CONTROL {
                            let _ = self.clear_sender.send(true);
                        } else {
                            let _ = self.motion_sender.send(value);
                        }
                    }
                    _ => {}
                }
            }
            Mode::Visual { .. } => todo!("work on visual mode for file key press"),
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
        let size = self.size;
        if let Some(buffer) = self.current_buffer_mut() {
            match &buffer.buffer_type {
                BufferType::Directory => {
                    buffer.insert_key_dir(key);
                }
                _ => {
                    buffer.insert_key_file(key, size);
                }
            }
        }
    }

    pub fn paste(&mut self, text: String) {
        if let Some(buffer) = self.current_buffer_mut() {
            let _ = &buffer.paste(text);
        }
    }

    // NOTE: word movements

    // TODO: needs to recalculate the viewpoint
    pub fn go_to_line(&mut self, line_idx: usize) {
        // adjust for 0 indexing
        let line_idx = line_idx.checked_sub(1).unwrap_or(0);

        let has_line = self
            .current_buffer()
            .and_then(|b| b.lines.rope.get_line(line_idx))
            .is_some();

        if has_line {
            let size = self.size;
            if let Some(buffer) = self.current_buffer_mut() {
                // move to that line, move cursor.y and ptr_y correctly
                buffer.cursor.current.0 = 0;
                buffer.cursor.current.1 = 0;
                buffer.ptr_y = 0;

                if line_idx > size.1.into() {
                    // account for UI
                    buffer.cursor.current.1 = size.1.into();
                    buffer.cursor.current.1.checked_sub(3).unwrap_or(0);
                    buffer.ptr_y = line_idx.checked_sub(size.1.into()).unwrap_or(0) + 3;
                } else {
                    buffer.cursor.current.1 = line_idx;
                }

                // update cursor x accordingly
                // FIX: temp solution
                buffer.cursor.current.0 = 0;
            }
        }
    }


    // TODO: 
    // create a simple way to parse motions from the string output
    pub fn parse(&mut self, motion: Vec<String>) -> Result<u32, &str> {
        self.send(format!("recv: {}", make_motion_string(&motion)));

        if motion.len() == 0 {
            unreachable!("motion len 0");
        } else if motion.len() == 1 {
            match motion.get(0) {
                Some(m) => {
                    self.motion_func(m);
                },
                None => {
                    self.send(format!("Error in parse: m == None"));
                },
            }
        } else {
            let mut number = 1;
            let mut m = String::from("");
            let mut f = String::from("");

            for (i, b) in motion.iter().enumerate() {
                if i == motion.len() - 1 {
                    m = b.to_string();
                } else if b.parse::<usize>().is_ok() {
                    number *= b.parse().unwrap_or(1);
                } else {
                    f = b.to_string();
                }
            }

            for _ in 0..number {
                self.motion_func(&m);
                self.action_func(&f, &String::from(" "));
            }
        }


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
                        self.new_buffer(&".".to_string());
                    },
                    CommandKey::BufCount => {
                        // sent message to count of opened buffers
                        let message = String::from(format!("{} open buffers", self.buffers.len()));
                        self.set_message(Some(message))
                    },
                }
            },
            None => {}
        }
    }

    pub fn motion_func(&mut self, key: &String) {
        let size = self.size;
        match key.as_str() {
            ":" => self.change_mode(Mode::Command),
            "j" => {
                if let Some(buffer) = self.current_buffer_mut() {
                    buffer.move_down(size)
                }
            }
            "k" => {
                if let Some(buffer) = self.current_buffer_mut() {
                    buffer.move_up()
                }
            }
            "h" => {
                if let Some(buffer) = self.current_buffer_mut() {
                    buffer.move_left()
                }
            }
            "l" => {
                if let Some(buffer) = self.current_buffer_mut() {
                    buffer.move_right()
                }
            }
            "i" => self.change_mode(Mode::Insert),
            "v" => {
                // TODO: grab current x and y coord, do I even need x and y or can i use byte
                self.change_mode(Mode::Visual { start: 0, end: 0 })
            }
            "a" => {
                if let Some(buffer) = self.current_buffer_mut() {
                    buffer.change_mode(Mode::Insert);
                    buffer.move_right();
                }
            }
            "O" => {
                if let Some(buffer) = self.current_buffer_mut() {
                    buffer.new_line_above(size);
                }
            }
            "o" => {
                if let Some(buffer) = self.current_buffer_mut() {
                    buffer.new_line_below(size);
                }
            }
            "w" => {
                if let Some(buffer) = self.current_buffer_mut() {
                    buffer.move_next_word(size)
                }
            }
            "b" => {
                if let Some(buffer) = self.current_buffer_mut() {
                    buffer.move_back_word(size)
                }
            }
            "e" => {
                if let Some(buffer) = self.current_buffer_mut() {
                    buffer.move_end_word(size)
                }
            }
            "0" => {
                if let Some(buffer) = self.current_buffer_mut() {
                    buffer.move_begin_of_line()
                }
            }
            "$" => {
                if let Some(buffer) = self.current_buffer_mut() {
                    buffer.move_end_of_line()
                }
            }
            "I" => {
                if let Some(buffer) = self.current_buffer_mut() {
                    buffer.change_mode(Mode::Insert);
                    buffer.move_begin_of_line();
                }
            }
            "A" => {
                if let Some(buffer) = self.current_buffer_mut() {
                    buffer.change_mode(Mode::Insert);
                    buffer.move_end_of_line();
                }
            }
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

    pub fn new_buffer(&mut self, path: &String){
        // WARN: check for possible errors that can return

        // TODO: fix this, determine what should be passed
        let buf = Buffer::new(path, self.size);

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
        if let Some(message) = new_mes {
            self.message = Some(message.clone());
            self.send(format!("Set Message {}", message.clone()));
        } else {
            self.message = None;
            self.send(format!("Message was cleared"));
        }
    }

    // NOTE: saving functions

    // TODO: make this safer by reading permissions
    pub fn save(&mut self) -> String {
        // FIX: too much extra memory
        self.current_buffer()
            .map_or(String::from("No buffer to save"), |b| b.save())
    }

    // NOTE: functions for logging

    pub fn send(&mut self, message: String) {
        match &mut self.logger {
            Some(stream) => {
                let _ = stream.write(message.as_bytes());
            },
            None => {}
        }
    }

    // NOTE: mouse functions
    pub fn handle_mouse(&mut self, mouse_event: MouseEvent) {
        if let Some(buffer) = self.current_buffer_mut() {
            buffer.mouse_handler(&mouse_event);
        }
    }

    // NOTE: window management
    // TODO: modify cursor location
    pub fn resize(&mut self, new_size: (u16, u16)) {
        self.size = new_size;

        for buffer in self.buffers.iter_mut() {
            buffer.resize(new_size);
        }
    }

    // TODO: Swap interface with frame interface
    // this function might just get deleted
    pub fn set_cursor(&self, f: &mut Frame<'_>) {
        if let Some(buffer) = self.current_buffer() {
            match &buffer.mode {
                Mode::Command => {
                    f.set_cursor(
                        (self.command.text.len() + 1).try_into().unwrap(),
                        f.size().height,
                    );
                }
                _ => {
                    f.set_cursor(
                        (buffer.cursor.current.0 + X_OFFSET).try_into().unwrap(),
                        (buffer.cursor.current.1).try_into().unwrap(),
                    );
                }
            };
        }
    }

    pub async fn next_motion(&mut self) -> Result<Vec<String>> {
        let event = self.motion_listener.recv().await.ok_or(color_eyre::eyre::eyre!("Unable to get action"));
        event
    }
}

impl <'a> Editor {
    // TODO: create for buffer
    // move to buffer to handle more logic
    pub fn buffer_display(&self) -> (Paragraph<'a>, Paragraph<'a>) {
        self.current_buffer()
            .map_or((Paragraph::new(""), Paragraph::new("")), |b| b.ui())
    }
}

impl Drop for Editor {
    fn drop(&mut self) {
        drop(self.clear_sender.to_owned());
        drop(self.motion_sender.to_owned());
        match std::mem::replace(&mut self.logger, None) {
            Some(stream) => {
                drop(stream);
            },
            None => {},
        }
    }
}

fn make_motion_string(input: &Vec<String>) -> String {
    let mut s = String::new();
    for i in input {
        s.push_str(i);
    }
    s
}


#[test]
fn test_make_motion_string(){
    assert_eq!(make_motion_string(&vec!["45".to_string(), "d".to_string(), "7".to_string(), "j".to_string()]), String::from("45d7j"));
    assert_eq!(make_motion_string(&vec!["a".to_string(), "b".to_string(), "c".to_string(), "123".to_string(), "j".to_string()]), String::from("abc123j"));
    assert_eq!(make_motion_string(&vec!["123".to_string(), "a".to_string()]), String::from("123a"));
    assert_eq!(make_motion_string(&vec!["j".to_string()]), String::from("j"));
    assert_eq!(make_motion_string(&vec![":".to_string()]), String::from(":"));
    assert_eq!(make_motion_string(&vec!["2".to_string(), ":".to_string()]), String::from("2:"));
}
