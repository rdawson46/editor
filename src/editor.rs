use crate::{
    buffer::{Buffer, BufferType, Mode},
    command::{Command, CommandKey},
    motion::{MotionBuffer, MotionHandler},
    X_OFFSET,
    // window::Window
};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers, MouseEvent, MouseEventKind, MouseButton};
use color_eyre::eyre::Result;
use std::{
    io::Write,
    net::TcpStream,
    usize,
};
use ratatui::{
    prelude::{Alignment, Style},
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
    },
    task::JoinHandle,
};

macro_rules! current_buf {
    ($e: expr) => {
        $e.buffers[$e.buf_ptr]
    };
}

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
    pub motion_listener: UnboundedReceiver<MotionBuffer>,
    pub motion_task: Option<JoinHandle<()>>,
}

impl Editor {
    pub fn new() -> Result<Editor> {
        // port address for logger
        let port = match std::env::args().nth(2) {
            Some(value) => value,
            None => "".to_string()
        };


        let (motion_buffer_sender, motion_buffer_listener) = mpsc::unbounded_channel(); 

        // motion sender is for key events
        let (mut motion, motion_sender, clear_sender) = MotionHandler::new(motion_buffer_sender);

        let motion_task = tokio::spawn(async move {
            loop {
                motion.listen().await;
            }
        });

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
                        motion_task: Some(motion_task),
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
            motion_task: Some(motion_task),
            motion_sender,
            clear_sender,
        });
    }

    pub fn change_mode(&mut self, mode: Mode) {
        current_buf!(self).change_mode(mode);

        match mode {
            Mode::Insert | Mode::Command => {
                self.set_message(None);
                self.command.clear();
            }
            _ => (),
        }
    }

    // NOTE: display functions
    pub fn mode_display(&mut self) -> (Paragraph, Option<Paragraph>) {
        match &current_buf!(self).mode {
            Mode::Insert => {
                (Paragraph::new("-- Insert --").block(Block::default().borders(Borders::TOP)), None)
            },
            Mode::Normal => {
                let motion_str = "".to_string();

                /*
                   match &self.motion.number {
                   Some(value) => motion_str.push_str(value.clone().as_str()),
                   None => {}
                   }

                   match &self.motion.action {
                   Some(value) => motion_str.push_str(value.clone().as_str()),
                   None => {}
                   }
                   */

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
            },
            Mode::Visual{..} => todo!("impl visual mode for ui"),
        }
    }

    // NOTE: event functions
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
                        let _ = current_buf!(self).open(&file_name);
                    },
                    KeyCode::Char(value) => {
                        if value == 'c' && key.modifiers == KeyModifiers::CONTROL {
                            let _ = self.clear_sender.send(true);
                        } else {
                            let _ = self.motion_sender.send(value);
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
                    },
                    _ => {}
                }
            },
            Mode::Visual{..} => todo!("work on visual mode for file key press")
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

    pub fn paste(&mut self, text: String) {
        let _ = &current_buf!(self).paste(text);
    }

    // NOTE: word movements

    // TODO: needs to recalculate the viewpoint
    pub fn go_to_line(&mut self, line_idx: usize) {
        // adjust for 0 indexing
        let line_idx = line_idx.checked_sub(1).unwrap_or(0);

        let slice = current_buf!(self).lines.rope.get_line(line_idx);

        if let Some(_slice) = slice {
            // move to that line, move cursor.y and ptr_y correctly
            current_buf!(self).cursor.current.0 = 0;
            current_buf!(self).cursor.current.1 = 0;
            current_buf!(self).ptr_y = 0;

            if line_idx > self.size.1.into() {
                // account for UI
                current_buf!(self).cursor.current.1 = self.size.1.into();
                current_buf!(self).cursor.current.1.checked_sub(3).unwrap_or(0);
                current_buf!(self).ptr_y = line_idx.checked_sub(self.size.1.into()).unwrap_or(0) + 3;
            } else {
                current_buf!(self).cursor.current.1 = line_idx;
            }

            // update cursor x accordingly
            // FIX: temp solution
            current_buf!(self).cursor.current.0 = 0;
        }
    }


    // NOTE: motion parsing function
    // might have to be async for timming
    //  could possile use channels for this
    //  might need an action function
    //  will probably remove returned result
    pub fn parse(&mut self, motion_buffer: MotionBuffer) -> Result<u32, &str> {
        let count = match motion_buffer.number{
            Some(value) => value.parse::<u32>().unwrap_or(0),
            None => 1,
        };

        let motion = match motion_buffer.motion {
            Some(value) => value.clone(),
            None => "".to_string(),
        };

        // TODO: figure out how have these commands run
        let action = match motion_buffer.action {
            Some(value) => value.clone(),
            None => "".to_string(),
        };

        let action_args = match motion_buffer.action_arg {
            Some(value) => value.clone(),
            None => "".to_string(),
        };

        for _ in 0..count {
            // perform action then move cursor
            self.motion_func(&motion);
            self.action_func(&action, &action_args);

            match &current_buf!(self).mode {
                Mode::Normal => {},
                _ => break,
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
        match key.as_str() {
            ":" => self.change_mode(Mode::Command),
            "j" => current_buf!(self).move_down(self.size),
            "k" => current_buf!(self).move_up(),
            "h" => current_buf!(self).move_left(),
            "l" => current_buf!(self).move_right(),
            "i" => self.change_mode(Mode::Insert),
            "v" => {
                // TODO: grab current x and y coord, do I even need x and y or can i use byte
                self.change_mode(Mode::Visual{start: 0, end: 0})
            }
            "a" => {
                current_buf!(self).change_mode(Mode::Insert);
                current_buf!(self).move_right();
            },
            "O" => {
                current_buf!(self).new_line_above(self.size);
            },
            "o" => {
                current_buf!(self).new_line_below(self.size);
            },
            "w" => current_buf!(self).move_next_word(self.size),
            "b" => current_buf!(self).move_back_word(self.size),
            "e" => current_buf!(self).move_end_word(self.size),
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
        return current_buf!(self).save();
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
        self.send(format!("mouse event: {:?}", mouse_event));
        current_buf!(self).mouse_handler(&mouse_event);
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
        match &current_buf!(self).mode {
            Mode::Command => {
                f.set_cursor(
                    (self.command.text.len() + 1).try_into().unwrap(),
                    f.size().height
                );
            },
            _ => {
                f.set_cursor(
                    (current_buf!(self).cursor.current.0 + X_OFFSET).try_into().unwrap(),
                    (current_buf!(self).cursor.current.1).try_into().unwrap()
                );
            }
        };
    }

    pub async fn next_motion(&mut self) -> Result<MotionBuffer> {
        let event = self.motion_listener.recv().await.ok_or(color_eyre::eyre::eyre!("Unable to get action"));
        event
    }
}

impl <'a> Editor {
    // TODO: create for buffer
    // move to buffer to handle more logic
    pub fn buffer_display(&self) -> (Paragraph<'a>, Paragraph<'a>) {
        current_buf!(self).ui()
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


        match std::mem::replace(&mut self.motion_task, None) {
            Some(handle) => {
                handle.abort();
            },
            None => {},
        }
    }
}
