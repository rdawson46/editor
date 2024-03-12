use std::rc::Rc;
use std::usize;
use crate::Event;
use crate::Tui;
use crate::X_OFFSET;
use crate::command::CommandKey;
use crossterm::event::KeyCode;
use crossterm::event::KeyModifiers;
use crate::editor::{Editor, Mode};
use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Padding, Paragraph},
    Frame,
};

// TODO: replace editor.ptr with y_ptr and x_ptr for horizontal scrolling
// fix for buffer types

fn get_layouts(f: &mut Frame<'_>) -> (Rc<[Rect]>, Rc<[Rect]>) {
    // wrapper_layout[0] is for the text and line numbers
    // wrapper_layout[1] is for the command view
    let wrapper_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints(vec![
                     Constraint::Min(1),
                     Constraint::Length(2)
        ])
        .split(f.size());

    let num_text_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![
                     Constraint::Length(X_OFFSET - 1),
                     Constraint::Min(1)
        ])
        .split(wrapper_layout[0]);

    return (wrapper_layout, num_text_layout);
}



pub fn ui(f: &mut Frame<'_>, editor: &mut Editor){
    let (wrapper_layout, num_text_layout) = get_layouts(f);
    
    editor.size = (num_text_layout[1].width, num_text_layout[1].height);

    // loop to make text for line nums and file text
    let mut line_nums = "".to_string();
    let mut text_string = "".to_string();

    for (i, line) in editor.buffer.lines.lines.iter().skip(editor.buffer.ptr_y.into()).enumerate() {
        if i > usize::from(editor.buffer.ptr_y + editor.size.1) {
            break;
        }

        let mut i_str: String;
        let current_line = usize::from(editor.buffer.cursor.current.1);

        if current_line != i {
            if current_line > i {
                i_str = (current_line - i).to_string();
            } else{
                i_str = (i - current_line).to_string();
            }

        } else {
            i_str = (editor.buffer.ptr_y + editor.buffer.cursor.current.1 + 1).to_string();
            if i_str.len() <= 2 {
                i_str.push(' ');
            }
        }

        i_str.push_str("\n\r");

        for char in i_str.chars() {
            line_nums.push(char);
        }

        for char in line.text.chars() {
            text_string.push(char);
        }

        text_string.push('\r');
        text_string.push('\n');
    }

    let (status, motion) = editor.mode_display();

    match motion {
        Some(motion) => {
            let status_motion = Layout::default()
                .direction(Direction::Horizontal)
                .constraints(vec![
                             Constraint::Percentage(50),
                             Constraint::Percentage(50)
                ])
                .split(wrapper_layout[1]);

            f.render_widget(status, status_motion[0]);
            f.render_widget(motion, status_motion[1]);
        },
        None => {
            f.render_widget(status, wrapper_layout[1]);
        }
    }

    f.render_widget(Paragraph::new(line_nums)
                    .alignment(ratatui::layout::Alignment::Right)
                    .style(Style::default().fg(Color::DarkGray)),
                    num_text_layout[0]);
    
    f.render_widget(Paragraph::new(text_string)
                    .block(Block::default()
                       .padding(Padding::new(1, 0, 0, 0))),
                    num_text_layout[1]);
}



//  TODO: fix how modes switch, ???
pub fn update(editor: &mut Editor, event: Event, tui: &mut Tui){
    match event {
        Event::Init => {},
        Event::Quit => {},
        Event::Error => {},
        Event::Closed => {},
        Event::Tick => {},
        Event::Render => {},
        Event::FocusGained => {},
        Event::FocusLost => {},
        Event::Paste(_) => {},
        Event::Key(key) => {

            // TODO: add movable cursor with arrow keys
            match editor.buffer.mode {
                Mode::Insert => {
                    // TODO: fix for directory
                    editor.insert_key(key);
                },

                Mode::Command => {
                    match key.code {
                        KeyCode::Char(value) => {
                            if value == 'c' && key.modifiers == KeyModifiers::CONTROL {
                                editor.change_mode(Mode::Normal);
                            } else {
                                editor.command.text.push(value);
                            }
                        },
                        KeyCode::Esc => {
                            editor.change_mode(Mode::Normal);
                        }
                        KeyCode::Enter => {
                            let command = editor.command.confirm();

                            match command {
                                Some(command) => {
                                    match command {
                                        CommandKey::Save => editor.save(),
                                        CommandKey::Quit => editor.should_quit = true,
                                        CommandKey::Line(number) => {
                                            editor.go_to_line(number);
                                        },
                                        CommandKey::SaveAndQuit => {
                                            editor.save();
                                            editor.should_quit = true;
                                        },
                                        CommandKey::History => todo!(),
                                        CommandKey::Logger => {
                                            // TODO: finish this up

                                            let output = match &editor.logger {
                                                Some(socket) => {
                                                    let addr = socket.local_addr().unwrap().to_string();
                                                    format!("Binded to {}", addr)
                                                },
                                                None => "Not Connected".to_string()
                                            };
                                            editor.message = Some(output);
                                        },
                                        CommandKey::Send(message) => {
                                            match &editor.logger {
                                                Some(socket) => {
                                                    let _ = socket.send(message.as_bytes());
                                                },
                                                None => {}
                                            }
                                        }
                                    }
                                },
                                None => {}
                            }
                            editor.change_mode(Mode::Normal);
                        },
                        KeyCode::Backspace => {
                            if editor.command.text.len() > 0 {
                                // TODO: add movable cursor
                                editor.command.text.pop();
                            } else {
                                editor.change_mode(Mode::Normal);
                            }
                        },
                        _ => {}
                    }
                },

                Mode::Normal => {
                    match key.code {
                        KeyCode::Char(value) => {
                            if value == 's' && key.modifiers == KeyModifiers::CONTROL {
                                editor.save();
                            } else if value == 'c' && key.modifiers == KeyModifiers::CONTROL {
                                editor.motion.clear();
                            } else {

                                let res = editor.motion.push(value);

                                match res {
                                    Some(_) => {
                                        let _ = editor.parse();
                                        editor.motion.clear();
                                    },
                                    None => {}
                                }
                            }
                        },
                        _ => {}
                    }
                },
            }
        },
        Event::Mouse(_) => {},
        Event::Resize(x, y) => {
            tui.size = (x, y);
        },
    }
}
